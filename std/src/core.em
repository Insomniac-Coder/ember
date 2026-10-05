## `std.core` — Part XV's prelude module.
##
## Part IV §8's standard interfaces, spelled as ordinary interfaces exactly as
## that section writes them. `Option`, `Result`, `Array` and `String` are still
## compiler-known (Part XX.1 makes them so "until Phase 2's generics let the
## standard library write them"); this module holds what can already be
## written, and grows as the compiler does.

import std.mem

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

## Part IV §8 — a receiver-less interface member is an associated function,
## invoked through the implementing type as `T.default()`.
pub interface Default:
    fn default() -> Self

pub interface Clone:
    fn clone(self) -> Self

## Part IV §8 — the operator interfaces. `a + b` on a type that is not a
## number calls its `Add.add(a, b)` (`[TYP-21]`); `Rhs` is the right operand's
## type, `Self` unless the implementation names another (`Mul[Vec4]`), and
## `Output` the result's. `a += b` calls `AddAssign.add_assign` where a type
## implements it, and is `a = a + b` otherwise. A number's operators are
## built in; the `extend` blocks at the end of this file say which of these
## interfaces each number type meets, so generic code bounded by them
## (`T: Add[Output = T]`) takes numbers too (ODR-040).
pub interface Add[Rhs = Self]:
    type Output
    fn add(self, rhs: Rhs) -> Output

pub interface Sub[Rhs = Self]:
    type Output
    fn sub(self, rhs: Rhs) -> Output

pub interface Mul[Rhs = Self]:
    type Output
    fn mul(self, rhs: Rhs) -> Output

pub interface Div[Rhs = Self]:
    type Output
    fn div(self, rhs: Rhs) -> Output

pub interface FloorDiv[Rhs = Self]:
    type Output
    fn floordiv(self, rhs: Rhs) -> Output

pub interface Rem[Rhs = Self]:
    type Output
    fn rem(self, rhs: Rhs) -> Output

pub interface Pow[Rhs = Self]:
    type Output
    fn pow(self, rhs: Rhs) -> Output

pub interface BitAnd[Rhs = Self]:
    type Output
    fn bitand(self, rhs: Rhs) -> Output

pub interface BitOr[Rhs = Self]:
    type Output
    fn bitor(self, rhs: Rhs) -> Output

pub interface BitXor[Rhs = Self]:
    type Output
    fn bitxor(self, rhs: Rhs) -> Output

pub interface Shl[Rhs = Self]:
    type Output
    fn shl(self, rhs: Rhs) -> Output

pub interface Shr[Rhs = Self]:
    type Output
    fn shr(self, rhs: Rhs) -> Output

## Unary `-` and `~`.
pub interface Neg:
    type Output
    fn neg(self) -> Output

pub interface Not:
    type Output
    fn not(self) -> Output

pub interface AddAssign[Rhs = Self]:
    fn add_assign(mut self, rhs: Rhs)

pub interface SubAssign[Rhs = Self]:
    fn sub_assign(mut self, rhs: Rhs)

pub interface MulAssign[Rhs = Self]:
    fn mul_assign(mut self, rhs: Rhs)

pub interface DivAssign[Rhs = Self]:
    fn div_assign(mut self, rhs: Rhs)

pub interface FloorDivAssign[Rhs = Self]:
    fn floordiv_assign(mut self, rhs: Rhs)

pub interface RemAssign[Rhs = Self]:
    fn rem_assign(mut self, rhs: Rhs)

pub interface PowAssign[Rhs = Self]:
    fn pow_assign(mut self, rhs: Rhs)

pub interface BitAndAssign[Rhs = Self]:
    fn bitand_assign(mut self, rhs: Rhs)

pub interface BitOrAssign[Rhs = Self]:
    fn bitor_assign(mut self, rhs: Rhs)

pub interface BitXorAssign[Rhs = Self]:
    fn bitxor_assign(mut self, rhs: Rhs)

pub interface ShlAssign[Rhs = Self]:
    fn shl_assign(mut self, rhs: Rhs)

pub interface ShrAssign[Rhs = Self]:
    fn shr_assign(mut self, rhs: Rhs)

## `a[i]` reads through `Index.index`, and is written in place through
## `IndexMut.index_mut`; `a[i] = v` calls `IndexSet.index_set` where the type
## has it, which is how a `Map` inserts (`[STD-17]`).
pub interface Index[Idx]:
    type Output
    fn index(self, i: Idx) -> ref Output

pub interface IndexMut[Idx]: Index[Idx]:
    fn index_mut(mut self, i: Idx) -> ref mut Output

pub interface IndexSet[Idx, V]:
    fn index_set(mut self, i: Idx, owned v: V)

## An `Array` and a `MutSpan` are read and written in place, and a `Span` read,
## by their built-in indexing, which is their `Index` and `IndexMut`
## (`[STD-17]`; an `Array` needs no `IndexSet`).
extend[T] Array[T] implements Index[int], IndexMut[int]:
    type Output = T

extend[T] Span[T] implements Index[int]:
    type Output = T

extend[T] MutSpan[T] implements Index[int], IndexMut[int]:
    type Output = T

## `[STD-19]` (G8-4) — a count of items: what `len`, `count` and `position` give, in an
## iterator's `Count`, and `enumerate`'s numbers, in its `Position`. An iterator over stored items counts in `int`, which no array, view, string
## or collection outgrows; a range of wide numbers counts in a type that holds every count it can
## have (the owner's design, `docs/proposals/G8-4-counts-and-positions.md`). These are the
## operations the adapters count with.
pub interface ItemCount: Ord + Copy:
    ## The count of `n` items, which is not negative.
    fn of(n: int) -> Self
    ## The count `n`, which fits: how counts of different types meet.
    fn of_wide(n: u256) -> Self
    ## This count as a `u256`, which holds every count; a negative number has none.
    fn to_wide(self) -> u256

    ## The count `n`, or this type's largest where `n` is past it.
    fn at_most(n: u256) -> Self

    ## This count in the count type `C`, which holds it.
    fn as_count[C: ItemCount](self) -> C:
        return C.of_wide(self.to_wide())
    ## Whether this is the type's largest value, which has no next one.
    fn at_top(self) -> bool
    ## This count and `other`'s items together.
    fn plus(self, other: Self) -> Self
    ## This count less `other`, which is not more than it.
    fn less(self, other: Self) -> Self
    ## Whether this count is more than `n` items.
    fn exceeds(self, n: int) -> bool
    ## This count divided by `k`, which is positive, rounded down.
    fn div_by(self, k: int) -> Self
    ## What is left of this count after dividing it by `k`, which is positive.
    fn rem_by(self, k: int) -> int
    ## This count as an `int`; one past `int.MAX` panics.
    fn to_int(self) -> int

extend i64 implements ItemCount:
    fn of(n: int) -> i64:
        return n

    fn at_most(n: u256) -> i64:
        if n > (int.MAX as u256):
            return int.MAX
        return n as i64

    fn of_wide(n: u256) -> i64:
        if n > (int.MAX as u256):
            panic(f"{n} does not fit an int")
        return n as i64

    fn to_wide(self) -> u256:
        if self < 0:
            panic(f"{self} is not a count")
        return self as u256

    fn at_top(self) -> bool:
        return self == 9223372036854775807

    fn plus(self, other: i64) -> i64:
        return self + other

    fn less(self, other: i64) -> i64:
        return self - other

    fn exceeds(self, n: int) -> bool:
        return self > n

    fn div_by(self, k: int) -> i64:
        return self // k

    fn rem_by(self, k: int) -> int:
        return self % k

    fn to_int(self) -> int:
        return self

## G8-4 — what a `..` range of 64-bit numbers counts in.
extend u64 implements ItemCount:
    fn of(n: int) -> u64:
        if n < 0:
            panic(f"a count cannot be negative: {n}")
        return n as u64

    fn at_most(n: u256) -> u64:
        if n > (u64.MAX as u256):
            return u64.MAX
        return n as u64

    fn of_wide(n: u256) -> u64:
        if n > (u64.MAX as u256):
            panic(f"{n} does not fit a u64")
        return n as u64

    fn to_wide(self) -> u256:
        return self

    fn at_top(self) -> bool:
        return self == 18446744073709551615

    fn plus(self, other: u64) -> u64:
        return self + other

    fn less(self, other: u64) -> u64:
        return self - other

    fn exceeds(self, n: int) -> bool:
        return n < 0 or self > (n as u64)

    fn div_by(self, k: int) -> u64:
        return self // (k as u64)

    fn rem_by(self, k: int) -> int:
        return (self % (k as u64)) as int

    fn to_int(self) -> int:
        if self > 9223372036854775807:
            panic(f"a count of {self} does not fit an int")
        return self as int

## G8-4 — what a `..` range of 128-bit numbers counts in, and a `..=` one of 64-bit numbers.
extend u128 implements ItemCount:
    fn of(n: int) -> u128:
        if n < 0:
            panic(f"a count cannot be negative: {n}")
        return n as u128

    fn at_most(n: u256) -> u128:
        if n > (u128.MAX as u256):
            return u128.MAX
        return n as u128

    fn of_wide(n: u256) -> u128:
        if n > (u128.MAX as u256):
            panic(f"{n} does not fit a u128")
        return n as u128

    fn to_wide(self) -> u256:
        return self

    fn at_top(self) -> bool:
        return self == u128.MAX

    fn plus(self, other: u128) -> u128:
        return self + other

    fn less(self, other: u128) -> u128:
        return self - other

    fn exceeds(self, n: int) -> bool:
        return n < 0 or self > (n as u128)

    fn div_by(self, k: int) -> u128:
        return self // (k as u128)

    fn rem_by(self, k: int) -> int:
        return (self % (k as u128)) as int

    fn to_int(self) -> int:
        if self > 9223372036854775807:
            panic(f"a count of {self} does not fit an int")
        return self as int

## G8-4 — what `enumerate` numbers a range of 64-bit numbers with.
extend i128 implements ItemCount:
    fn of(n: int) -> i128:
        return n

    fn at_most(n: u256) -> i128:
        if n > (i128.MAX as u256):
            return i128.MAX
        return n as i128

    fn of_wide(n: u256) -> i128:
        if n > (i128.MAX as u256):
            panic(f"{n} does not fit an i128")
        return n as i128

    fn to_wide(self) -> u256:
        if self < 0:
            panic(f"{self} is not a count")
        return self as u256

    fn at_top(self) -> bool:
        return self == i128.MAX

    fn plus(self, other: i128) -> i128:
        return self + other

    fn less(self, other: i128) -> i128:
        return self - other

    fn exceeds(self, n: int) -> bool:
        return self > (n as i128)

    fn div_by(self, k: int) -> i128:
        return self // (k as i128)

    fn rem_by(self, k: int) -> int:
        return (self % (k as i128)) as int

    fn to_int(self) -> int:
        if self > (int.MAX as i128) or self < (int.MIN as i128):
            panic(f"a number of {self} does not fit an int")
        return self as int

## G8-4 — what `enumerate` numbers a range of 128-bit numbers with (`i256`, the signed 256-bit
## count: a start may be negative).
extend i256 implements ItemCount:
    fn of(n: int) -> i256:
        return n

    ## A count past the largest `i256` has no number: it panics.
    fn at_most(n: u256) -> i256:
        return i256.of_wide(n)

    fn of_wide(n: u256) -> i256:
        number = n as i256
        if number < 0:
            panic(f"{n} does not fit an i256")
        return number

    fn to_wide(self) -> u256:
        if self < 0:
            panic(f"{self} is not a count")
        return self as u256

    ## Never: no number comes near its top.
    fn at_top(self) -> bool:
        return false

    fn plus(self, other: i256) -> i256:
        return self + other

    fn less(self, other: i256) -> i256:
        return self - other

    fn exceeds(self, n: int) -> bool:
        return self > (n as i256)

    fn div_by(self, k: int) -> i256:
        return count_divmod(self.to_wide(), k as u256).0 as i256

    fn rem_by(self, k: int) -> int:
        return count_divmod(self.to_wide(), k as u256).1 as int

    fn to_int(self) -> int:
        if self > (int.MAX as i256) or self < (int.MIN as i256):
            panic(f"a number of {self} does not fit an int")
        return self as int

## G8-4 — what a `..=` range of 128-bit numbers counts in (`u256`, the 256-bit count). It adds,
## subtracts and compares, so it divides by doubling.
extend u256 implements ItemCount:
    fn of(n: int) -> u256:
        if n < 0:
            panic(f"a count cannot be negative: {n}")
        return n as u256

    fn at_most(n: u256) -> u256:
        return n

    fn of_wide(n: u256) -> u256:
        return n

    fn to_wide(self) -> u256:
        return self

    ## Never: no count comes near its top.
    fn at_top(self) -> bool:
        return false

    fn plus(self, other: u256) -> u256:
        return self + other

    fn less(self, other: u256) -> u256:
        return self - other

    fn exceeds(self, n: int) -> bool:
        return n < 0 or self > (n as u256)

    fn div_by(self, k: int) -> u256:
        return count_divmod(self, k as u256).0

    fn rem_by(self, k: int) -> int:
        return count_divmod(self, k as u256).1 as int

    fn to_int(self) -> int:
        if self > (int.MAX as u256):
            panic(f"a count of {self} does not fit an int")
        return self as int

## `n` divided by `d`, which is positive, and what is left: the quotient by `2d` (`d + d` is not
## past `n`), doubled, and one more `d` when it is left.
fn count_divmod(n: u256, d: u256) -> (u256, u256):
    if n < d:
        return (0, n)
    if n - d < d:
        return (1, n - d)
    half = count_divmod(n, d + d)
    if half.1 < d:
        return (half.0 + half.0, half.1)
    return (half.0 + half.0 + 1, half.1 - d)

## Part IV §8's canonical associated-type iterator contract. Named standard
## iterators, including the Arena-backed collection and Span iterators,
## implement this interface rather than introducing a second iterator
## abstraction.
pub interface Iterator:
    type Item
    ## `[STD-19]` (G8-4) — what this iterator counts its items in: `int` unless it says otherwise.
    type Count: ItemCount = int
    ## And what `enumerate` numbers its items with.
    type Position: ItemCount = int
    fn next(mut self) -> Option[Item]

    ## `[STD-19]` (ODR-089) — the adapters. Each takes this iterator and
    ## gives a new one; nothing runs until the new one's `next` is called. A
    ## bad count panics here, where the bug is, never partway through a loop.

    ## At most `n` items.
    fn take(owned self, n: int) -> Take[Self]:
        if n < 0:
            panic(f"take({n}): a count cannot be negative")
        return Take(self, n)

    ## All but the first `n` items.
    fn skip(owned self, n: int) -> Skip[Self]:
        if n < 0:
            panic(f"skip({n}): a count cannot be negative")
        return Skip(self, n)

    ## The first item, then every `k`-th.
    fn step_by(owned self, k: int) -> StepBy[Self]:
        if k <= 0:
            panic(f"step_by({k}): the step must be positive")
        return StepBy(self, k - 1, false)

    ## Each item with its number, counting from `start` (`[STD-26]`).
    fn enumerate(owned self, start: int = 0) -> Enumerate[Self, Position]:
        return Enumerate(self, Position.of(start), false)

    ## Pairs of an item of each, until either runs out.
    fn zip[J: Iterator](owned self, other: J) -> Zip[Self, J]:
        return Zip(self, other)

    ## This iterator's items, then `other`'s, which are of the same type.
    fn chain[J: Iterator[Item = Item]](owned self, other: J) -> Chain[Self, J]:
        return Chain(self, other, false)

    ## What `f` makes of each item. The adapter holds `f` itself, not a
    ## pointer to it, so nothing is allocated and each call is direct
    ## (`[CLO-14]`).
    fn map[R, F: fn(Item) -> R](owned self, f: F) -> Map[Self, R, F]:
        return Map(self, f)

    ## The items `pred` holds for.
    fn filter[F: fn(Item) -> bool](owned self, pred: F) -> Filter[Self, F]:
        return Filter(self, pred)

    ## What `f` gives for each item it gives something for.
    fn filter_map[R, F: fn(Item) -> Option[R]](owned self, f: F) -> FilterMap[Self, R, F]:
        return FilterMap(self, f)

    ## The items before the first one `pred` does not hold for.
    fn take_while[F: fn(Item) -> bool](owned self, pred: F) -> TakeWhile[Self, F]:
        return TakeWhile(self, pred, false)

    ## The items from the first one `pred` does not hold for.
    fn skip_while[F: fn(Item) -> bool](owned self, pred: F) -> SkipWhile[Self, F]:
        return SkipWhile(self, pred, false)

    ## The items, each given to `f` first.
    fn inspect[F: fn(Item)](owned self, f: F) -> Inspect[Self, F]:
        return Inspect(self, f)

    ## The items of each iterable `f` makes of an item, one after another.
    fn flat_map[U: IntoIterator, F: fn(Item) -> U](owned self, f: F) -> FlatMap[Self, U, F]:
        return FlatMap(self, f, None)

    ## The items of each item, one after another, when the items are
    ## iterable themselves.
    fn flatten(owned self) -> Flatten[Self, Item] where Item: IntoIterator:
        return Flatten(self, None)

    ## These items, with `peek` to look at the next one without taking it
    ## (ODR-094).
    fn peekable(owned self) -> Peekable[Self]:
        return Peekable(self, None, false)

    ## `[STD-19]` — the consumers. Each takes this iterator and runs it.

    ## How many items there are.
    fn count(owned self) -> Count:
        n = Count.of(0)
        one = Count.of(1)
        for _ in self:
            n = n.plus(one)
        return n

    ## The last item, if any.
    fn last(owned self) -> Option[Item]:
        found: Option[Item] = None
        for x in self:
            found = Some(x)
        return found

    ## The item at index `n`, if there are that many.
    fn nth(owned self, n: int) -> Option[Item]:
        if n < 0:
            panic(f"nth({n}): an index cannot be negative")
        rest = self
        if rest.skip_front(Count.of(n)) > Count.of(0):
            return None
        return rest.next()

    ## `[STD-19]` (G8-4) — leaves out up to `n` items from the front, and gives how many of the `n`
    ## it could not (none when all went). A range does it at once, and the adapters that never
    ## change their items pass it down, so no gigantic run is passed one item at a time.
    fn skip_front(mut self, n: Count) -> Count:
        left = n
        zero = Count.of(0)
        one = Count.of(1)
        while left > zero:
            if self.next().is_none():
                return left
            left = left.less(one)
        return left

    ## `f(... f(f(init, a), b) ..., z)` over the items `a` to `z`.
    fn fold[B](owned self, init: B, f: fn(B, Item) -> B) -> B:
        acc = init
        for x in self:
            acc = f(acc, x)
        return acc

    ## Whether `pred` holds for some item; stops at the first.
    fn any(owned self, pred: fn(Item) -> bool) -> bool:
        for x in self:
            if pred(x):
                return true
        return false

    ## Whether `pred` holds for every item; stops at the first it does not.
    fn all(owned self, pred: fn(Item) -> bool) -> bool:
        for x in self:
            if not pred(x):
                return false
        return true

    ## The first item `pred` holds for.
    fn find(owned self, pred: fn(Item) -> bool) -> Option[Item]:
        for x in self:
            if pred(x):
                return Some(x)
        return None

    ## The index of the first item `pred` holds for.
    fn position(owned self, pred: fn(Item) -> bool) -> Option[Count]:
        at = Count.of(0)
        one = Count.of(1)
        for x in self:
            if pred(x):
                return Some(at)
            at = at.plus(one)
        return None

    ## `f` on each item, in order.
    fn for_each(owned self, f: fn(Item)):
        for x in self:
            f(x)

    ## The largest item; the last of equal largest ones. `None` when empty.
    fn max(owned self) -> Option[Item] where Item: Ord:
        best: Option[Item] = None
        for x in self:
            match best:
                Some(b):
                    if not (x < b):
                        best = Some(x)
                None:
                    best = Some(x)
        return best

    ## The smallest item; the first of equal smallest ones. `None` when empty.
    fn min(owned self) -> Option[Item] where Item: Ord:
        best: Option[Item] = None
        for x in self:
            match best:
                Some(b):
                    if x < b:
                        best = Some(x)
                None:
                    best = Some(x)
        return best

    ## The item whose `key` is largest; the last of equal ones.
    fn max_by_key[K: Ord](owned self, key: fn(Item) -> K) -> Option[Item]:
        best: Option[Item] = None
        best_key: Option[K] = None
        for x in self:
            k = key(x)
            match best_key:
                Some(b):
                    if not (k < b):
                        best = Some(x)
                        best_key = Some(k)
                None:
                    best = Some(x)
                    best_key = Some(k)
        return best

    ## The item whose `key` is smallest; the first of equal ones.
    fn min_by_key[K: Ord](owned self, key: fn(Item) -> K) -> Option[Item]:
        best: Option[Item] = None
        best_key: Option[K] = None
        for x in self:
            k = key(x)
            match best_key:
                Some(b):
                    if k < b:
                        best = Some(x)
                        best_key = Some(k)
                None:
                    best = Some(x)
                    best_key = Some(k)
        return best

    ## The largest item by `cmp`; the last of equal ones.
    fn max_by(owned self, cmp: fn(Item, Item) -> Ordering) -> Option[Item]:
        best: Option[Item] = None
        for x in self:
            match best:
                Some(b):
                    if cmp(x, b) != Ordering.Less:
                        best = Some(x)
                None:
                    best = Some(x)
        return best

    ## The smallest item by `cmp`; the first of equal ones.
    fn min_by(owned self, cmp: fn(Item, Item) -> Ordering) -> Option[Item]:
        best: Option[Item] = None
        for x in self:
            match best:
                Some(b):
                    if cmp(x, b) == Ordering.Less:
                        best = Some(x)
                None:
                    best = Some(x)
        return best

    ## `f(... f(f(a, b), c) ..., z)` over the items; `None` when empty.
    fn reduce(owned self, f: fn(Item, Item) -> Item) -> Option[Item]:
        acc: Option[Item] = None
        for x in self:
            match acc:
                Some(a):
                    acc = Some(f(a, x))
                None:
                    acc = Some(x)
        return acc

    ## The items, in order, in a new array.
    fn to_array(owned self) -> Array[Item]:
        out: Array[Item] = []
        for x in self:
            out.push(x)
        return out

    ## The items gathered into a collection `C` (ODR-094): any type that can
    ## be built from them (`FromIterator`), written as `collect[C]()` or taken
    ## from the type the call is expected to give (`a: Array[int] =
    ## it.collect()`).
    fn collect[C: FromIterator[Item]](owned self) -> C:
        return C.from_iter(self)

    ## The items as text, `sep` between each two (`[1, 2, 3]` with `", "` is
    ## `"1, 2, 3"`).
    fn join(owned self, sep: str) -> String where Item: Display:
        out = String()
        first = true
        for x in self:
            if not first:
                out += sep
            out += f"{x}"
            first = false
        return out

## `[CTL-1]` — what `for x in owned e:` consumes: `e.into_iter()` is the
## iterator, and the loop takes what its `next` gives, owned.
## `[STD-19]` (ODR-091) — an iterator that can run backwards: `next_back`
## gives the last item not yet given from either end, so with `next` the two
## ends meet and no item comes twice. `rev` is its: `rev` exists exactly
## where an iterator can run backwards. The ranges of integers and the
## element iterators of the views are ones, and so is each adapter over
## ones, where it can tell which item is last (below).
pub interface DoubleEndedIterator: Iterator:
    fn next_back(mut self) -> Option[Item]

    ## `[STD-19]` (G8-4) — leaves out up to `n` items from the back, and gives how many of the `n`
    ## it could not (none when all went). A range does it at once, and the adapters pass it down.
    fn skip_back(mut self, n: Count) -> Count:
        left = n
        zero = Count.of(0)
        one = Count.of(1)
        while left > zero:
            if self.next_back().is_none():
                return left
            left = left.less(one)
        return left

    ## `[STD-19]` (G8-4) — leaves out `n` items from the back and gives the one before them:
    ## `nth_back(Count.of(0))` is `next_back()`.
    fn nth_back(mut self, n: Count) -> Option[Item]:
        if self.skip_back(n) > Count.of(0):
            return None
        return self.next_back()

    ## The items, last first.
    fn rev(owned self) -> Rev[Self]:
        return Rev(self)

## `[STD-19]` (ODR-091) — an iterator that knows exactly how many items it has
## left (`len`), without giving them: the ranges of integers, the views'
## iterators, and the adapters over ones.
pub interface ExactSizeIterator: Iterator:
    fn len(self) -> Count

pub interface IntoIterator:
    type Item
    type Iter: Iterator[Item = Item]
    fn into_iter(owned self) -> Iter

## `[STD-19]` (ODR-094) — a collection that can be built from values handed
## to it one at a time, which is what `collect` asks of its target. `Array`,
## `Set`, `Map` (from key-value pairs) and `String` (from characters or from
## strings) are; so is any type of a program's that implements it.
pub interface FromIterator[T]:
    fn from_iter[I: Iterator[Item = T]](owned it: I) -> Self

## `[STD-19]` — the adapters `Iterator`'s methods above make. Each holds the
## iterator it wraps and gives what `next` says, as that method describes.
pub struct Take[I]:
    inner: I
    left: int

extend[I: Iterator] Take[I] implements Iterator:
    type Item = I.Item
    type Count = I.Count
    type Position = I.Position

    fn next(mut self) -> Option[I.Item]:
        if self.left <= 0:
            return None
        self.left -= 1
        return self.inner.next()

    fn skip_front(mut self, n: I.Count) -> I.Count:
        wanted = n
        if n.exceeds(self.left):
            wanted = I.Count.of(max(self.left, 0))
        missed = self.inner.skip_front(wanted)
        gone = wanted.less(missed)
        self.left -= gone.to_int()
        return n.less(gone)

pub struct Skip[I]:
    inner: I
    left: int

extend[I: Iterator] Skip[I] implements Iterator:
    type Item = I.Item
    type Count = I.Count
    type Position = I.Position

    fn next(mut self) -> Option[I.Item]:
        if self.left > 0:
            first = I.Count.of(self.left)
            self.left = 0
            if self.inner.skip_front(first) > I.Count.of(0):
                return None
        return self.inner.next()

    fn skip_front(mut self, n: I.Count) -> I.Count:
        if self.left > 0:
            first = I.Count.of(self.left)
            self.left = 0
            if self.inner.skip_front(first) > I.Count.of(0):
                return n
        return self.inner.skip_front(n)

pub struct StepBy[I]:
    inner: I
    ## The items left out between two given: the step less one.
    gap: int
    started: bool

extend[I: Iterator] StepBy[I] implements Iterator:
    type Item = I.Item
    type Count = I.Count
    type Position = I.Position

    fn next(mut self) -> Option[I.Item]:
        if self.started:
            left = self.gap
            while left > 0:
                left -= 1
                if self.inner.next().is_none():
                    return None
        self.started = true
        return self.inner.next()

## Item `k` is numbered `start + k`, so a number past `int`'s top panics, and
## only then; a `for` over it counts the same way (`[CTL-3b]`). `number` is the
## next item's, and `past` says the last one given was numbered `int.MAX`.
## G8-4 — the numbers are `P`s: the iterator's `Position`, `int` for stored items and wider for
## a range of 64- or 128-bit numbers, so every item has its number; or, where the range's numbers
## are written in place, the smallest type that holds them (the visible-numbers rule).
pub struct Enumerate[I, P]:
    inner: I
    number: P
    past: bool

extend[I: Iterator, P: ItemCount] Enumerate[I, P] implements Iterator:
    type Item = (P, I.Item)
    type Count = I.Count
    type Position = P

    fn next(mut self) -> Option[(P, I.Item)]:
        match self.inner.next():
            Some(x):
                at = self.number
                if self.past:
                    at = at.plus(P.of(1))
                elif at.at_top():
                    self.past = true
                else:
                    self.number = at.plus(P.of(1))
                return Some((at, x))
            None:
                return None

pub struct Zip[I, J]:
    a: I
    b: J

extend[I: Iterator, J: Iterator] Zip[I, J] implements Iterator:
    type Item = (I.Item, J.Item)
    type Count = I.Count
    type Position = I.Position

    fn next(mut self) -> Option[(I.Item, J.Item)]:
        match self.a.next():
            Some(x):
                match self.b.next():
                    Some(y):
                        return Some((x, y))
                    None:
                        return None
            None:
                return None

    fn skip_front(mut self, n: I.Count) -> I.Count:
        mine = self.a.skip_front(n).to_wide()
        wide = n.to_wide()
        theirs = J.Count.at_most(wide)
        missed = self.b.skip_front(theirs).to_wide() + (wide - theirs.to_wide())
        if missed > mine:
            return I.Count.of_wide(missed)
        return I.Count.of_wide(mine)

## `other` runs only once the first has run out (`done`), and the first is
## not asked again after it said `None`.
pub struct Chain[I, J]:
    first: I
    other: J
    done: bool

extend[I: Iterator, J: Iterator[Item = I.Item]] Chain[I, J] implements Iterator:
    type Item = I.Item
    ## G8-4 — the two sides' items together, stepping up by need: `int` and `int` stay `int`
    ## (stored items and small ranges cannot come near its top); otherwise one level above the
    ## wider side's count, `u64` -> `u128` -> `u256`, so every count of the two has its type.
    type Count = CountSum[I.Count, J.Count]
    ## And the numbers `enumerate` gives it, which hold both sides'.
    type Position = PositionJoin[I.Position, J.Position]

    fn next(mut self) -> Option[I.Item]:
        if not self.done:
            match self.first.next():
                Some(x):
                    return Some(x)
                None:
                    self.done = true
        return self.other.next()

    fn skip_front(mut self, n: CountSum[I.Count, J.Count]) -> CountSum[I.Count, J.Count]:
        left = n.to_wide()
        if not self.done:
            mine = I.Count.at_most(left)
            missed = self.first.skip_front(mine).to_wide()
            left = missed + (left - mine.to_wide())
            if left == 0:
                return n.less(n)
            self.done = true
        theirs = J.Count.at_most(left)
        left = self.other.skip_front(theirs).to_wide() + (left - theirs.to_wide())
        return left.as_count[CountSum[I.Count, J.Count]]()

## `[STD-19]` — `it.copied()` and `it.cloned()` over an iterator of
## references `ref T`: the values they reach, copied or cloned, so the items
## are `T`s a program may keep (`xs.iter().copied().to_array()`). The checker
## makes these for an iterator whose items are references (the methods are
## not written here because `Item` being `ref T` is not a bound it can state).
pub struct Copied[I, T]:
    inner: I

extend[T: Copy, I: Iterator[Item = ref T]] Copied[I, T] implements Iterator:
    type Item = T
    type Count = I.Count
    type Position = I.Position

    fn next(mut self) -> Option[T]:
        match self.inner.next():
            Some(r):
                return Some(r)
            None:
                return None

    fn skip_front(mut self, n: I.Count) -> I.Count:
        return self.inner.skip_front(n)

pub struct Cloned[I, T]:
    inner: I

extend[T: Clone, I: Iterator[Item = ref T]] Cloned[I, T] implements Iterator:
    type Item = T
    type Count = I.Count
    type Position = I.Position

    fn next(mut self) -> Option[T]:
        match self.inner.next():
            Some(r):
                return Some(r.clone())
            None:
                return None

## `[STD-19]` — the adapters that hold a callable: `map`, `filter`,
## `filter_map`, `take_while`, `skip_while` and `inspect`. Each keeps the
## callable it was given by value, as `[CLO-14]`'s bound makes it, so a
## lambda that borrows keeps its borrows, and the adapter is a view while it
## does (`[TYP-15]`).
pub struct Map[I, R, F]:
    inner: I
    f: F

extend[I: Iterator, R, F: fn(I.Item) -> R] Map[I, R, F] implements Iterator:
    type Item = R
    type Count = I.Count
    type Position = I.Position

    fn next(mut self) -> Option[R]:
        match self.inner.next():
            Some(x):
                return Some(self.f(x))
            None:
                return None

pub struct Filter[I, F]:
    inner: I
    pred: F

extend[I: Iterator, F: fn(I.Item) -> bool] Filter[I, F] implements Iterator:
    type Item = I.Item
    type Count = I.Count
    type Position = I.Position

    fn next(mut self) -> Option[I.Item]:
        while true:
            match self.inner.next():
                Some(x):
                    if self.pred(x):
                        return Some(x)
                None:
                    return None
        return None

pub struct FilterMap[I, R, F]:
    inner: I
    f: F

extend[I: Iterator, R, F: fn(I.Item) -> Option[R]] FilterMap[I, R, F] implements Iterator:
    type Item = R
    type Count = I.Count
    type Position = I.Position

    fn next(mut self) -> Option[R]:
        while true:
            match self.inner.next():
                Some(x):
                    match self.f(x):
                        Some(y):
                            return Some(y)
                        None:
                            pass
                None:
                    return None
        return None

pub struct TakeWhile[I, F]:
    inner: I
    pred: F
    done: bool

extend[I: Iterator, F: fn(I.Item) -> bool] TakeWhile[I, F] implements Iterator:
    type Item = I.Item
    type Count = I.Count
    type Position = I.Position

    fn next(mut self) -> Option[I.Item]:
        if self.done:
            return None
        match self.inner.next():
            Some(x):
                if self.pred(x):
                    return Some(x)
                self.done = true
                return None
            None:
                return None

pub struct SkipWhile[I, F]:
    inner: I
    pred: F
    started: bool

extend[I: Iterator, F: fn(I.Item) -> bool] SkipWhile[I, F] implements Iterator:
    type Item = I.Item
    type Count = I.Count
    type Position = I.Position

    fn next(mut self) -> Option[I.Item]:
        if self.started:
            return self.inner.next()
        while true:
            match self.inner.next():
                Some(x):
                    if not self.pred(x):
                        self.started = true
                        return Some(x)
                None:
                    return None
        return None

pub struct Inspect[I, F]:
    inner: I
    f: F

extend[I: Iterator, F: fn(I.Item)] Inspect[I, F] implements Iterator:
    type Item = I.Item
    type Count = I.Count
    type Position = I.Position

    fn next(mut self) -> Option[I.Item]:
        match self.inner.next():
            Some(x):
                self.f(x)
                return Some(x)
            None:
                return None

## `[STD-19]` — `flat_map(f)`: the items of what `f` makes of each item.
## `inner` walks the current one; when it runs out, the next item is made into
## the next one, and when the items run out, so does this.
pub struct FlatMap[I, U: IntoIterator, F]:
    outer: I
    f: F
    inner: Option[U.Iter]

extend[I: Iterator, U: IntoIterator, F: fn(I.Item) -> U] FlatMap[I, U, F] implements Iterator:
    type Item = U.Item
    ## G8-4 — every inner iterator's items together, stepping up by need: the level that holds the
    ## outer count times the inner one (an `int` by an `int` stays an `int`; `u256` the limit).
    type Count = CountProduct[I.Count, U.Iter.Count]
    ## And the numbers `enumerate` gives them.
    type Position = PositionOf[CountProduct[I.Count, U.Iter.Count]]

    fn next(mut self) -> Option[U.Item]:
        while true:
            match self.inner.as_mut():
                Some(it):
                    match it.next():
                        Some(x):
                            return Some(x)
                        None:
                            pass
                None:
                    pass
            match self.outer.next():
                Some(x):
                    self.inner = Some((self.f)(x).into_iter())
                None:
                    return None
        return None

## `[STD-19]` — `flatten()`: the items of each item, as `flat_map` gives them
## with nothing made of the items first. `U` is the items' type, named so its
## bound can be stated.
pub struct Flatten[I, U: IntoIterator]:
    outer: I
    inner: Option[U.Iter]

extend[U: IntoIterator, I: Iterator[Item = U]] Flatten[I, U] implements Iterator:
    type Item = U.Item
    ## G8-4 — every inner iterator's items together, stepping up by need: the level that holds the
    ## outer count times the inner one (an `int` by an `int` stays an `int`; `u256` the limit).
    type Count = CountProduct[I.Count, U.Iter.Count]
    ## And the numbers `enumerate` gives them.
    type Position = PositionOf[CountProduct[I.Count, U.Iter.Count]]

    fn next(mut self) -> Option[U.Item]:
        while true:
            match self.inner.as_mut():
                Some(it):
                    match it.next():
                        Some(x):
                            return Some(x)
                        None:
                            pass
                None:
                    pass
            match self.outer.next():
                Some(x):
                    self.inner = Some(x.into_iter())
                None:
                    return None
        return None

## `[STD-19]` (ODR-094) — `peekable()`: the items of the iterator it wraps,
## with the next one taken early by `peek` and kept (`looked`) until `next`
## gives it.
pub struct Peekable[I: Iterator]:
    inner: I
    peeked: Option[I.Item]
    looked: bool

extend[I: Iterator] Peekable[I] implements Iterator:
    type Item = I.Item
    type Count = I.Count
    type Position = I.Position

    fn next(mut self) -> Option[I.Item]:
        if self.looked:
            self.looked = false
            return self.peeked.take()
        return self.inner.next()

extend[I: Iterator] Peekable[I]:
    ## The next item, left where it is: the following `next` gives it.
    pub fn peek(mut self) -> Option[ref I.Item]:
        if not self.looked:
            self.peeked = self.inner.next()
            self.looked = true
        return self.peeked.as_ref()

    ## The next item, left where it is, to change in place.
    pub fn peek_mut(mut self) -> Option[ref mut I.Item]:
        if not self.looked:
            self.peeked = self.inner.next()
            self.looked = true
        return self.peeked.as_mut()

    ## The next item when `pred` holds for it; otherwise it stays.
    pub fn next_if[F: fn(ref I.Item) -> bool](mut self, pred: F) -> Option[I.Item]:
        take = false
        match self.peek():
            Some(x):
                take = pred(x)
            None:
                pass
        if take:
            return self.next()
        return None

extend[T: Eq, I: Iterator[Item = T]] Peekable[I]:
    ## The next item when it equals `expected`; otherwise it stays.
    pub fn next_if_eq(mut self, expected: T) -> Option[T]:
        take = false
        match self.peek():
            Some(x):
                take = x == expected
            None:
                pass
        if take:
            return self.next()
        return None

## `[STD-19]` (ODR-091) — an iterator run backwards: its `next` is the one it
## wraps's `next_back`, and the other way round.
pub struct Rev[I]:
    inner: I

extend[I: DoubleEndedIterator] Rev[I] implements Iterator:
    type Item = I.Item
    type Count = I.Count
    type Position = I.Position

    fn next(mut self) -> Option[I.Item]:
        return self.inner.next_back()

    fn skip_front(mut self, n: I.Count) -> I.Count:
        return self.inner.skip_back(n)

extend[I: DoubleEndedIterator] Rev[I] implements DoubleEndedIterator:
    fn next_back(mut self) -> Option[I.Item]:
        return self.inner.next()

    fn skip_back(mut self, n: I.Count) -> I.Count:
        return self.inner.skip_front(n)

extend[I: DoubleEndedIterator + ExactSizeIterator] Rev[I] implements ExactSizeIterator:
    fn len(self) -> I.Count:
        return self.inner.len()

## ODR-091 — which adapters run backwards, and know their length. One whose
## last item depends on how many there are (`take`, `skip`, `step_by`,
## `enumerate`, `zip`) runs backwards when the iterator below it knows its
## length; `copied`, `cloned` and `chain` run backwards when theirs do.
extend[I: ExactSizeIterator] Take[I] implements ExactSizeIterator:
    fn len(self) -> I.Count:
        left = self.inner.len()
        if left.exceeds(self.left):
            return I.Count.of(max(self.left, 0))
        return left

## G8-4 — the items after the first `left` are left out from the back at once (`nth_back`), so a
## range too long for an `int` to count gives its first items backwards too.
extend[I: DoubleEndedIterator + ExactSizeIterator] Take[I] implements DoubleEndedIterator:
    fn next_back(mut self) -> Option[I.Item]:
        if self.left <= 0:
            return None
        left = self.inner.len()
        keep = self.left
        self.left -= 1
        if left.exceeds(keep):
            return self.inner.nth_back(left.less(I.Count.of(keep)))
        return self.inner.next_back()

    fn skip_back(mut self, n: I.Count) -> I.Count:
        if self.left <= 0:
            return n
        wanted = n
        if n.exceeds(self.left):
            wanted = I.Count.of(self.left)
        held = self.inner.len()
        if held.exceeds(self.left):
            self.inner.skip_back(held.less(I.Count.of(self.left)))
        missed = self.inner.skip_back(wanted)
        gone = wanted.less(missed)
        self.left -= gone.to_int()
        return n.less(gone)

extend[I: ExactSizeIterator] Skip[I] implements ExactSizeIterator:
    fn len(self) -> I.Count:
        left = self.inner.len()
        if left.exceeds(self.left):
            return left.less(I.Count.of(self.left))
        return I.Count.of(0)

extend[I: DoubleEndedIterator + ExactSizeIterator] Skip[I] implements DoubleEndedIterator:
    fn next_back(mut self) -> Option[I.Item]:
        if not self.inner.len().exceeds(self.left):
            return None
        return self.inner.next_back()

    fn skip_back(mut self, n: I.Count) -> I.Count:
        held = self.inner.len()
        if not held.exceeds(self.left):
            return n
        room = held.less(I.Count.of(self.left))
        wanted = n
        if n > room:
            wanted = room
        missed = self.inner.skip_back(wanted)
        return n.less(wanted.less(missed))

## The items a `step_by(k)` gives are at `0, k, 2k, …` of those left before
## the first, then at `k - 1, 2k - 1, …` after it (`gap` is `k - 1`).
extend[I: ExactSizeIterator] StepBy[I] implements ExactSizeIterator:
    fn len(self) -> I.Count:
        left = self.inner.len()
        step = self.gap + 1
        if self.started:
            return left.div_by(step)
        if not left.exceeds(0):
            return I.Count.of(0)
        one = I.Count.of(1)
        return one.plus(left.less(one).div_by(step))

extend[I: DoubleEndedIterator + ExactSizeIterator] StepBy[I] implements DoubleEndedIterator:
    fn next_back(mut self) -> Option[I.Item]:
        left = self.inner.len()
        step = self.gap + 1
        extra = 0
        if self.started:
            if not left.exceeds(step - 1):
                return None
            extra = left.rem_by(step)
        else:
            if not left.exceeds(0):
                return None
            extra = left.less(I.Count.of(1)).rem_by(step)
        if extra > 0:
            return self.inner.nth_back(I.Count.of(extra))
        return self.inner.next_back()

extend[I: ExactSizeIterator, P: ItemCount] Enumerate[I, P] implements ExactSizeIterator:
    fn len(self) -> I.Count:
        return self.inner.len()

## The last item left is numbered the next one's number plus the items
## between.
extend[I: DoubleEndedIterator + ExactSizeIterator, P: ItemCount] Enumerate[I, P] implements DoubleEndedIterator:
    fn next_back(mut self) -> Option[(P, I.Item)]:
        left = self.inner.len()
        match self.inner.next_back():
            Some(x):
                at = self.number.plus(P.of_wide(left.to_wide() - 1))
                if self.past:
                    at = at.plus(P.of(1))
                return Some((at, x))
            None:
                return None

    ## The items left out are numbered as `next_back` numbers them, the greatest first, so a
    ## number past the type's top panics as it would item by item (`[CTL-3b]`, D-457).
    fn skip_back(mut self, n: I.Count) -> I.Count:
        left = self.inner.len()
        if n.exceeds(0) and left.exceeds(0):
            at = self.number.plus(P.of_wide(left.to_wide() - 1))
            if self.past:
                at.plus(P.of(1))
        return self.inner.skip_back(n)

extend[I: ExactSizeIterator, J: ExactSizeIterator] Zip[I, J] implements ExactSizeIterator:
    fn len(self) -> I.Count:
        mine = self.a.len()
        theirs = self.b.len().to_wide()
        if theirs < mine.to_wide():
            return I.Count.of_wide(theirs)
        return mine

## G8-4 — the longer side's extra items left out at once, so both have as many left.
extend[I: DoubleEndedIterator + ExactSizeIterator, J: DoubleEndedIterator + ExactSizeIterator] Zip[I, J]:
    fn even_up(mut self):
        mine = self.a.len().to_wide()
        theirs = self.b.len().to_wide()
        if mine > theirs:
            self.a.skip_back(I.Count.of_wide(mine - theirs))
        if theirs > mine:
            self.b.skip_back(J.Count.of_wide(theirs - mine))

## The longer is cut to the shorter's length from the back first, so the
## pairs are the ones `next` would make.
extend[I: DoubleEndedIterator + ExactSizeIterator, J: DoubleEndedIterator + ExactSizeIterator] Zip[I, J] implements DoubleEndedIterator:
    fn next_back(mut self) -> Option[(I.Item, J.Item)]:
        self.even_up()
        match self.a.next_back():
            Some(x):
                match self.b.next_back():
                    Some(y):
                        return Some((x, y))
                    None:
                        return None
            None:
                return None

    fn skip_back(mut self, n: I.Count) -> I.Count:
        self.even_up()
        missed = self.a.skip_back(n)
        self.b.skip_back(J.Count.at_most(n.to_wide()))
        return missed

extend[I: ExactSizeIterator, J: ExactSizeIterator[Item = I.Item]] Chain[I, J] implements ExactSizeIterator:
    fn len(self) -> CountSum[I.Count, J.Count]:
        rest = self.other.len().as_count[CountSum[I.Count, J.Count]]()
        if self.done:
            return rest
        return self.first.len().as_count[CountSum[I.Count, J.Count]]().plus(rest)

## Backwards the second runs first; the first is not asked once it said
## `None` from the front.
extend[I: DoubleEndedIterator, J: DoubleEndedIterator[Item = I.Item]] Chain[I, J] implements DoubleEndedIterator:
    fn next_back(mut self) -> Option[I.Item]:
        match self.other.next_back():
            Some(x):
                return Some(x)
            None:
                if self.done:
                    return None
                return self.first.next_back()

    fn skip_back(mut self, n: CountSum[I.Count, J.Count]) -> CountSum[I.Count, J.Count]:
        left = n.to_wide()
        theirs = J.Count.at_most(left)
        left = self.other.skip_back(theirs).to_wide() + (left - theirs.to_wide())
        if left > 0 and not self.done:
            mine = I.Count.at_most(left)
            left = self.first.skip_back(mine).to_wide() + (left - mine.to_wide())
        return left.as_count[CountSum[I.Count, J.Count]]()

extend[T: Copy, I: ExactSizeIterator[Item = ref T]] Copied[I, T] implements ExactSizeIterator:
    fn len(self) -> I.Count:
        return self.inner.len()

extend[T: Copy, I: DoubleEndedIterator[Item = ref T]] Copied[I, T] implements DoubleEndedIterator:
    fn next_back(mut self) -> Option[T]:
        match self.inner.next_back():
            Some(r):
                return Some(r)
            None:
                return None

    fn skip_back(mut self, n: I.Count) -> I.Count:
        return self.inner.skip_back(n)

extend[T: Clone, I: ExactSizeIterator[Item = ref T]] Cloned[I, T] implements ExactSizeIterator:
    fn len(self) -> I.Count:
        return self.inner.len()

extend[T: Clone, I: DoubleEndedIterator[Item = ref T]] Cloned[I, T] implements DoubleEndedIterator:
    fn next_back(mut self) -> Option[T]:
        match self.inner.next_back():
            Some(r):
                return Some(r.clone())
            None:
                return None

## `[CTL-3]` (ODR-027) — the range types. `a..b` is a `Range`, `a..=b` a
## `RangeInclusive`, `a..` a `RangeFrom` and `..b` a `RangeTo`. Each is a plain
## value, `Copy` when its bound is, with public bounds. A `for` over one of the
## first three with integer bounds is a counted loop over a copy of its bounds
## (`[CTL-3b]`), so the range itself is left as it was.
@derive(Copy)
pub struct Range[T]:
    pub start: T
    pub end: T

@derive(Copy)
pub struct RangeInclusive[T]:
    pub start: T
    pub end: T

@derive(Copy)
pub struct RangeFrom[T]:
    pub start: T

@derive(Copy)
pub struct RangeTo[T]:
    pub end: T

## `[STD-19]`, `[CTL-3b]` — a range of integers is `Iterable`, as Python's
## `range` is: `r.iter()` counts through a copy of its bounds, so the range is
## left as it was, and `(a..b).skip(1)` is `(a..b).iter().skip(1)`. A `for`
## over a chain of adapters on one is a counted loop.
extend[T: Integer] Range[T]:
    pub fn iter(self) -> RangeIter[T]:
        return RangeIter(self.start, self.end)

extend[T: Integer] RangeInclusive[T]:
    pub fn iter(self) -> RangeInclusiveIter[T]:
        return RangeInclusiveIter(self.start, self.end, self.end < self.start)

extend[T: Integer] RangeFrom[T]:
    pub fn iter(self) -> RangeFromIter[T]:
        return RangeFromIter(self.start, false)

## `[CTL-1]` (D-444) — a range is iterable, and consumed as a value its
## items are its `iter()`'s: `flat_map(fn(x: int) => 0..x)`.
extend[T: Integer] Range[T] implements IntoIterator:
    type Item = T
    type Iter = RangeIter[T]

    fn into_iter(owned self) -> RangeIter[T]:
        return self.iter()

extend[T: Integer] RangeInclusive[T] implements IntoIterator:
    type Item = T
    type Iter = RangeInclusiveIter[T]

    fn into_iter(owned self) -> RangeInclusiveIter[T]:
        return self.iter()

extend[T: Integer] RangeFrom[T] implements IntoIterator:
    type Item = T
    type Iter = RangeFromIter[T]

    fn into_iter(owned self) -> RangeFromIter[T]:
        return self.iter()

## The values from `at` up to, not including, `end`.
pub struct RangeIter[T]:
    at: T
    end: T

extend[T: Integer] RangeIter[T] implements Iterator:
    type Item = T
    type Count = T.SpanCount
    type Position = T.Position

    fn next(mut self) -> Option[T]:
        if self.at < self.end:
            here = self.at
            self.at = here.successor()
            return Some(here)
        return None

    fn skip_front(mut self, n: T.SpanCount) -> T.SpanCount:
        if self.at < self.end:
            gap = self.at.distance_to(self.end)
            if gap > n:
                self.at = self.at.forward_by(n)
                return T.SpanCount.of(0)
            self.at = self.end
            return n.less(gap)
        return n

## ODR-091 — backwards from `end`, which moves down to meet `at`; G8-4 — `nth_back` moves it
## past `n` values at once.
extend[T: Integer] RangeIter[T] implements DoubleEndedIterator:
    fn next_back(mut self) -> Option[T]:
        if self.at < self.end:
            self.end = self.end.predecessor()
            return Some(self.end)
        return None

    fn skip_back(mut self, n: T.SpanCount) -> T.SpanCount:
        if self.at < self.end:
            gap = self.at.distance_to(self.end)
            if gap > n:
                self.end = self.end.back_by(n)
                return T.SpanCount.of(0)
            self.end = self.at
            return n.less(gap)
        return n

extend[T: Integer] RangeIter[T] implements ExactSizeIterator:
    fn len(self) -> T.SpanCount:
        if self.at < self.end:
            return self.at.distance_to(self.end)
        return T.SpanCount.of(0)

## The values from `at` up to and including `end`; `done` once `end` is
## given, so a range ending at the type's top never steps past it.
pub struct RangeInclusiveIter[T]:
    at: T
    end: T
    done: bool

extend[T: Integer] RangeInclusiveIter[T] implements Iterator:
    type Item = T
    type Count = T.FullCount
    type Position = T.Position

    fn next(mut self) -> Option[T]:
        if self.done:
            return None
        here = self.at
        if here < self.end:
            self.at = here.successor()
        else:
            self.done = true
        return Some(here)

    fn skip_front(mut self, n: T.FullCount) -> T.FullCount:
        if self.done:
            return n
        count = T.widen_count(self.at.distance_to(self.end)).plus(T.FullCount.of(1))
        if count > n:
            self.at = self.at.forward_by(T.narrow_count(n))
            return T.FullCount.of(0)
        self.done = true
        return n.less(count)

## ODR-091 — backwards from `end`; `done` once the two meet, so a range
## starting at the type's bottom never steps below it.
extend[T: Integer] RangeInclusiveIter[T] implements DoubleEndedIterator:
    fn next_back(mut self) -> Option[T]:
        if self.done:
            return None
        here = self.end
        if self.at < here:
            self.end = here.predecessor()
        else:
            self.done = true
        return Some(here)

    ## G8-4 — `end` moved past `n` values at once; an `n` below the count leaves `at` at least.
    fn skip_back(mut self, n: T.FullCount) -> T.FullCount:
        if self.done:
            return n
        count = T.widen_count(self.at.distance_to(self.end)).plus(T.FullCount.of(1))
        if count > n:
            self.end = self.end.back_by(T.narrow_count(n))
            return T.FullCount.of(0)
        self.done = true
        return n.less(count)

extend[T: Integer] RangeInclusiveIter[T] implements ExactSizeIterator:
    fn len(self) -> T.FullCount:
        if self.done:
            return T.FullCount.of(0)
        return T.widen_count(self.at.distance_to(self.end)).plus(T.FullCount.of(1))

## The values from `at` on, the type's top included; the value after the top is an overflow
## (`[TYP-8]`), which panics when it is asked for, as in a `for` over `a..` (`[CTL-3]`, D-526).
## `given` says `at` has been given.
pub struct RangeFromIter[T]:
    at: T
    given: bool

extend[T: Integer] RangeFromIter[T] implements Iterator:
    type Item = T
    ## G8-4 — it gives the values up to the type's top at most.
    type Count = T.FullCount
    type Position = T.Position

    fn next(mut self) -> Option[T]:
        if self.given:
            self.at = self.at.successor()
        self.given = true
        return Some(self.at)

    ## G8-4 — the values left out at once; leaving out the value after the top panics, as asking
    ## for it would.
    fn skip_front(mut self, n: T.FullCount) -> T.FullCount:
        if not n.exceeds(0):
            return n
        if self.given:
            self.at = self.at.successor()
        self.given = true
        left = self.at.count_to_top()
        if n > left:
            self.at = self.at.forward_by(T.narrow_count(left.less(T.FullCount.of(1))))
            self.at = self.at.successor()
        self.at = self.at.forward_by(T.narrow_count(n.less(T.FullCount.of(1))))
        return T.FullCount.of(0)

## Part IV §8 also declares `Hash`, `Display`, `Debug`, and the operator
## interfaces; they remain staged with their dependent surface.

## `[ERR-4]` (ODR-025) — the `Option` and `Result` methods that take a
## function. `x.map(f)` is a call of `option_map(x, f)`: the compiler routes the
## method here, so each is an ordinary generic function whose callback is
## checked, inferred and borrow-checked like any other. Each consumes the
## receiver and calls `f` at most once, before it returns; a payload passed to
## `f` is moved in (`owned`), except `filter`'s, which it must give back.

fn option_map[T, U](owned o: Option[T], f: fn(owned T) -> U) -> Option[U]:
    match owned o:
        Some(v):
            return Some(f(v))
        None:
            return None

fn option_and_then[T, U](owned o: Option[T], f: fn(owned T) -> Option[U]) -> Option[U]:
    match owned o:
        Some(v):
            return f(v)
        None:
            return None

fn option_filter[T](owned o: Option[T], f: fn(T) -> bool) -> Option[T]:
    match owned o:
        Some(v):
            if f(v):
                return Some(v)
            return None
        None:
            return None

fn option_or_else[T](owned o: Option[T], f: fn() -> Option[T]) -> Option[T]:
    match owned o:
        Some(v):
            return Some(v)
        None:
            return f()

fn option_unwrap_or_else[T](owned o: Option[T], f: fn() -> T) -> T:
    match owned o:
        Some(v):
            return v
        None:
            return f()

fn option_ok_or_else[T, E](owned o: Option[T], f: fn() -> E) -> Result[T, E]:
    match owned o:
        Some(v):
            return Ok(v)
        None:
            return Err(f())

fn result_map[T, E, U](owned r: Result[T, E], f: fn(owned T) -> U) -> Result[U, E]:
    match owned r:
        Ok(v):
            return Ok(f(v))
        Err(e):
            return Err(e)

fn result_map_err[T, E, F](owned r: Result[T, E], f: fn(owned E) -> F) -> Result[T, F]:
    match owned r:
        Ok(v):
            return Ok(v)
        Err(e):
            return Err(f(e))

fn result_and_then[T, E, U](owned r: Result[T, E], f: fn(owned T) -> Result[U, E]) -> Result[U, E]:
    match owned r:
        Ok(v):
            return f(v)
        Err(e):
            return Err(e)

fn result_or_else[T, E, F](owned r: Result[T, E], f: fn(owned E) -> Result[T, F]) -> Result[T, F]:
    match owned r:
        Ok(v):
            return Ok(v)
        Err(e):
            return f(e)

fn result_unwrap_or_else[T, E](owned r: Result[T, E], f: fn(owned E) -> T) -> T:
    match owned r:
        Ok(v):
            return v
        Err(e):
            return f(e)

## `[TYP-36]` — the table's `Default` column: zero, `0.0`, `false`, `'\0'`,
## empty text, an empty `Array` and `None`.

extend i8 implements Default:
    fn default() -> i8:
        return 0

extend i16 implements Default:
    fn default() -> i16:
        return 0

extend i32 implements Default:
    fn default() -> i32:
        return 0

extend i64 implements Default:
    fn default() -> i64:
        return 0

extend isize implements Default:
    fn default() -> isize:
        return 0

extend u8 implements Default:
    fn default() -> u8:
        return 0

extend u16 implements Default:
    fn default() -> u16:
        return 0

extend u32 implements Default:
    fn default() -> u32:
        return 0

extend u64 implements Default:
    fn default() -> u64:
        return 0

extend usize implements Default:
    fn default() -> usize:
        return 0

extend i128 implements Default:
    fn default() -> i128:
        return 0

extend u128 implements Default:
    fn default() -> u128:
        return 0

extend f16 implements Default:
    fn default() -> f16:
        return 0.0

extend f32 implements Default:
    fn default() -> f32:
        return 0.0

extend f64 implements Default:
    fn default() -> f64:
        return 0.0

extend bool implements Default:
    fn default() -> bool:
        return false

extend char implements Default:
    fn default() -> char:
        return '\0'

extend str implements Default:
    fn default() -> str:
        return ""

## The characters, in order (ODR-094).
extend String implements FromIterator[char]:
    fn from_iter[I: Iterator[Item = char]](owned it: I) -> String:
        out = String()
        for c in it:
            out.push(c)
        return out

## The strings, one after another (ODR-094).
extend String implements FromIterator[String]:
    fn from_iter[I: Iterator[Item = String]](owned it: I) -> String:
        out = String()
        for s in it:
            out += s
        return out

extend String implements Default:
    fn default() -> String:
        return String.from("")

extend[T] Array[T] implements Default:
    fn default() -> Array[T]:
        return []

extend[T] Option[T] implements Default:
    fn default() -> Option[T]:
        return None

## `[ERR-4]` — `take` and `replace`, written in Ember with `mem.replace`.
extend[T] Option[T]:
    ## The payload, leaving `None`.
    pub fn take(mut self) -> Option[T]:
        return mem.replace(self, None)

    ## The payload, leaving `Some(value)`.
    pub fn replace(mut self, owned value: T) -> Option[T]:
        return mem.replace(self, Some(value))

## `[STD-15]` — the `Array` methods written in Ember (`[GRM-34]`). A method
## the compiler knows by a name comes first (`[TYP-24]`); these are found
## after it, and one a program never calls is not emitted (`[COST-1]`).
## `for x in owned xs:` (`[CTL-1]`): the elements in order, owned. The array
## is reversed once and popped, so each element is taken without a copy, and
## those left when a loop ends early are dropped with the iterator.
pub struct ArrayIntoIter[T]:
    rest: Array[T]

    pub fn next(mut self) -> Option[T]:
        return self.rest.pop()

extend[T] ArrayIntoIter[T] implements Iterator:
    type Item = T

## The items in order (ODR-094).
extend[T] Array[T] implements FromIterator[T]:
    fn from_iter[I: Iterator[Item = T]](owned it: I) -> Array[T]:
        out: Array[T] = []
        for x in it:
            out.push(x)
        return out

extend[T] Array[T] implements IntoIterator:
    type Item = T
    type Iter = ArrayIntoIter[T]

    fn into_iter(owned self) -> ArrayIntoIter[T]:
        rest = self
        rest.reverse()
        return ArrayIntoIter(rest)

extend[T] Array[T]:
    ## Keeps the elements `keep` is true of, in their order.
    pub fn retain(mut self, keep: fn(T) -> bool):
        kept = 0
        for i in range(self.len()):
            if keep(self[i]):
                self.swap(kept, i)
                kept += 1
        self.truncate(kept)

    ## Sorts by `cmp`, which says how two elements order. Stable: elements
    ## `cmp` finds equal keep their order (ODR-031).
    pub fn sort_by(mut self, cmp: fn(T, T) -> Ordering):
        order = stable_order(self, cmp)
        self.apply_order(order)

    ## Sorts by `key(x)`, calling `key` once for each element, as Python's
    ## `key=` does. Stable (ODR-031).
    pub fn sort_by_key[K: Ord](mut self, key: fn(T) -> K):
        keys: Array[K] = []
        for i in range(self.len()):
            keys.push(key(self[i]))
        order = stable_order(keys, fn(a, b) => a.cmp(b))
        self.apply_order(order)

    ## `[STD-15]` (ODR-068) — mutable references to the elements at `i` and
    ## `j` at once, or `None` when either index is out of range or they are
    ## the same: the mutable view's `get_pair_mut`.
    pub fn get_pair_mut(mut self, i: int, j: int) -> Option[(ref mut T, ref mut T)]:
        return self.as_mut_span().get_pair_mut(i, j)

    ## Puts the element at `order[i]` at `i`, following each cycle of the
    ## permutation with `swap`: no element is copied or moved out, so any `T`
    ## sorts.
    fn apply_order(mut self, mut order: Array[int]):
        for start in range(self.len()):
            at = start
            while order[at] != start:
                next = order[at]
                self.swap(at, next)
                order[at] = at
                at = next
            order[at] = at

extend[T] MutSpan[T]:
    ## `[SPN-5]` (ODR-068) — mutable references to the elements at `i` and
    ## `j` at once, or `None` when either index is out of range or they are
    ## the same. The references come from `split_at`'s two halves, so they
    ## are disjoint before either exists.
    pub fn get_pair_mut(mut self, i: int, j: int) -> Option[(ref mut T, ref mut T)]:
        n = self.len()
        if i < 0 or j < 0 or i >= n or j >= n or i == j:
            return None
        if i < j:
            (low, high) = self.split_at(j)
            return Some((ref mut low[i], ref mut high[0]))
        (low, high) = self.split_at(i)
        return Some((ref mut high[0], ref mut low[j]))

## `[STR-5]` — element-wise equality through `T: Eq`. The checker routes a
## comparison of sequences here when the element type holds a written `eq`.
extend[T: Eq] Span[T]:
    fn eq_elements(self, other: Span[T]) -> bool:
        if self.len() != other.len():
            return false
        for i in range(self.len()):
            if self[i] != other[i]:
                return false
        return true

extend[T: Eq] Array[T]:
    ## Drops each element equal to the one kept before it, so a sorted array
    ## keeps one of each value.
    pub fn dedup(mut self):
        if self.len() < 2:
            return
        kept = 1
        for i in range(1, self.len()):
            if self[i] != self[kept - 1]:
                self.swap(kept, i)
                kept += 1
        self.truncate(kept)

extend[T: Ord] Array[T]:
    ## In a sorted array: `Ok` of an index holding `x`, or `Err` of the index
    ## where `x` would go to keep the array sorted.
    pub fn binary_search(self, x: T) -> Result[int, int]:
        lo = 0
        hi = self.len()
        while lo < hi:
            mid = (lo + hi) // 2
            match self[mid].cmp(x):
                Ordering.Less: lo = mid + 1
                Ordering.Equal: return Ok(mid)
                Ordering.Greater: hi = mid
        return Err(lo)

    ## `sort` for an element type with an `Ord` of its own. The compiler's
    ## `sort` orders numbers and text itself and routes the rest here (D-256).
    fn sort_ord(mut self):
        order = stable_order(self, fn(a, b) => a.cmp(b))
        self.apply_order(order)

extend[T: Ord + Clone] Array[T]:
    ## `sorted` for what the compiler's own `sorted` does not take: elements
    ## with an `Ord` of their own, or that are not `Copy` (D-256).
    fn sorted_ord(self) -> Array[T]:
        out = self.clone()
        out.sort_ord()
        return out

## `[STD-15]` — the order a stable sort puts `xs` in, as indices: a bottom-up
## merge that takes from the right run only when `cmp` finds its element
## less. Each pass writes every index once, whatever `cmp` answers, so an
## inconsistent order can only permute the elements, never lose or repeat one
## (`[HASH-3]`).
fn stable_order[T](xs: Array[T], cmp: fn(T, T) -> Ordering) -> Array[int]:
    n = xs.len()
    order: Array[int] = []
    merged: Array[int] = []
    for i in range(n):
        order.push(i)
        merged.push(i)
    width = 1
    while width < n:
        lo = 0
        while lo < n:
            mid = min(lo + width, n)
            hi = min(lo + 2 * width, n)
            left = lo
            right = mid
            out = lo
            while left < mid and right < hi:
                if cmp(xs[order[right]], xs[order[left]]) == Ordering.Less:
                    merged[out] = order[right]
                    right += 1
                else:
                    merged[out] = order[left]
                    left += 1
                out += 1
            while left < mid:
                merged[out] = order[left]
                left += 1
                out += 1
            while right < hi:
                merged[out] = order[right]
                right += 1
                out += 1
            lo += 2 * width
        for i in range(n):
            order[i] = merged[i]
        width *= 2
    return order

extend[T: Display] Array[T]:
    ## The elements' text with `sep` between each two, as Python's
    ## `sep.join(xs)`.
    pub fn join(self, sep: str) -> String:
        out = String.from("")
        for i in range(self.len()):
            if i > 0:
                out += sep
            out += f"{self[i]}"
        return out

## ODR-040 — the number types and the operator interfaces. Each operand and
## each result is the type itself; the operators are the built-in ones, so an
## integer's `+` panics on overflow here too (`[TYP-8]`). An integer has no
## `/` (`E2240`) and a float no bit operators.

extend i8 implements Add, Sub, Mul, FloorDiv, Rem, Pow, Neg, Not, BitAnd, BitOr, BitXor, Shl, Shr, \
        AddAssign, SubAssign, MulAssign, FloorDivAssign, RemAssign, PowAssign, BitAndAssign, BitOrAssign, \
        BitXorAssign, ShlAssign, ShrAssign:
    type Output = i8

extend i16 implements Add, Sub, Mul, FloorDiv, Rem, Pow, Neg, Not, BitAnd, BitOr, BitXor, Shl, Shr, \
        AddAssign, SubAssign, MulAssign, FloorDivAssign, RemAssign, PowAssign, BitAndAssign, BitOrAssign, \
        BitXorAssign, ShlAssign, ShrAssign:
    type Output = i16

extend i32 implements Add, Sub, Mul, FloorDiv, Rem, Pow, Neg, Not, BitAnd, BitOr, BitXor, Shl, Shr, \
        AddAssign, SubAssign, MulAssign, FloorDivAssign, RemAssign, PowAssign, BitAndAssign, BitOrAssign, \
        BitXorAssign, ShlAssign, ShrAssign:
    type Output = i32

extend i64 implements Add, Sub, Mul, FloorDiv, Rem, Pow, Neg, Not, BitAnd, BitOr, BitXor, Shl, Shr, \
        AddAssign, SubAssign, MulAssign, FloorDivAssign, RemAssign, PowAssign, BitAndAssign, BitOrAssign, \
        BitXorAssign, ShlAssign, ShrAssign:
    type Output = i64

extend i128 implements Add, Sub, Mul, FloorDiv, Rem, Pow, Neg, Not, BitAnd, BitOr, BitXor, Shl, Shr, \
        AddAssign, SubAssign, MulAssign, FloorDivAssign, RemAssign, PowAssign, BitAndAssign, BitOrAssign, \
        BitXorAssign, ShlAssign, ShrAssign:
    type Output = i128

extend isize implements Add, Sub, Mul, FloorDiv, Rem, Pow, Neg, Not, BitAnd, BitOr, BitXor, Shl, Shr, \
        AddAssign, SubAssign, MulAssign, FloorDivAssign, RemAssign, PowAssign, BitAndAssign, BitOrAssign, \
        BitXorAssign, ShlAssign, ShrAssign:
    type Output = isize

extend u8 implements Add, Sub, Mul, FloorDiv, Rem, Pow, Neg, Not, BitAnd, BitOr, BitXor, Shl, Shr, \
        AddAssign, SubAssign, MulAssign, FloorDivAssign, RemAssign, PowAssign, BitAndAssign, BitOrAssign, \
        BitXorAssign, ShlAssign, ShrAssign:
    type Output = u8

extend u16 implements Add, Sub, Mul, FloorDiv, Rem, Pow, Neg, Not, BitAnd, BitOr, BitXor, Shl, Shr, \
        AddAssign, SubAssign, MulAssign, FloorDivAssign, RemAssign, PowAssign, BitAndAssign, BitOrAssign, \
        BitXorAssign, ShlAssign, ShrAssign:
    type Output = u16

extend u32 implements Add, Sub, Mul, FloorDiv, Rem, Pow, Neg, Not, BitAnd, BitOr, BitXor, Shl, Shr, \
        AddAssign, SubAssign, MulAssign, FloorDivAssign, RemAssign, PowAssign, BitAndAssign, BitOrAssign, \
        BitXorAssign, ShlAssign, ShrAssign:
    type Output = u32

extend u64 implements Add, Sub, Mul, FloorDiv, Rem, Pow, Neg, Not, BitAnd, BitOr, BitXor, Shl, Shr, \
        AddAssign, SubAssign, MulAssign, FloorDivAssign, RemAssign, PowAssign, BitAndAssign, BitOrAssign, \
        BitXorAssign, ShlAssign, ShrAssign:
    type Output = u64

extend u128 implements Add, Sub, Mul, FloorDiv, Rem, Pow, Neg, Not, BitAnd, BitOr, BitXor, Shl, Shr, \
        AddAssign, SubAssign, MulAssign, FloorDivAssign, RemAssign, PowAssign, BitAndAssign, BitOrAssign, \
        BitXorAssign, ShlAssign, ShrAssign:
    type Output = u128

extend usize implements Add, Sub, Mul, FloorDiv, Rem, Pow, Neg, Not, BitAnd, BitOr, BitXor, Shl, Shr, \
        AddAssign, SubAssign, MulAssign, FloorDivAssign, RemAssign, PowAssign, BitAndAssign, BitOrAssign, \
        BitXorAssign, ShlAssign, ShrAssign:
    type Output = usize

## G8-4 — the 256-bit counts have the operators a count has (`[TYP-42]`).
extend i256 implements Add, Sub, Neg, AddAssign, SubAssign:
    type Output = i256

extend u256 implements Add, Sub, Neg, AddAssign, SubAssign:
    type Output = u256

extend f16 implements Add, Sub, Mul, Div, FloorDiv, Rem, Pow, Neg, \
        AddAssign, SubAssign, MulAssign, DivAssign, FloorDivAssign, RemAssign, PowAssign:
    type Output = f16

extend f32 implements Add, Sub, Mul, Div, FloorDiv, Rem, Pow, Neg, \
        AddAssign, SubAssign, MulAssign, DivAssign, FloorDivAssign, RemAssign, PowAssign:
    type Output = f32

extend f64 implements Add, Sub, Mul, Div, FloorDiv, Rem, Pow, Neg, \
        AddAssign, SubAssign, MulAssign, DivAssign, FloorDivAssign, RemAssign, PowAssign:
    type Output = f64

## -- `NonZero` (`[STD-4]`) -----------------------------------------------------

## The integer types, which `NonZero` and the range iterators take. The
## interface is private, so no other type can join them, and its method is
## std's own.
interface Integer: Hash + Default + Ord + Copy:
    ## `[STD-19]` (G8-4) — what a `..` range of these numbers counts its values in, the
    ## smallest type that holds every count it can have: `int` for numbers of 8, 16 or 32
    ## bits, `u64` for 64 bits, `u128` for 128 bits.
    type SpanCount: ItemCount
    ## And a `..=` range, which can hold one value more: `int`, `u128`, `u256`.
    type FullCount: ItemCount
    ## What `enumerate` numbers such a range with, from a start that may be negative: `int`,
    ## `i128`, `i256`.
    type Position: ItemCount

    ## The next integer; past the type's top it panics, as `+ 1` does.
    fn successor(self) -> Self

    ## The integer before; past the type's bottom it panics, as `- 1` does.
    fn predecessor(self) -> Self

    ## How many integers from this one up to, not including, `end`, which is
    ## not below it: a `..` range's length.
    fn distance_to(self, end: Self) -> SpanCount

    ## The integer `n` before this one, which the type holds.
    fn back_by(self, n: SpanCount) -> Self

    ## The integer `n` after this one, which the type holds.
    fn forward_by(self, n: SpanCount) -> Self

    ## How many integers from this one up to the type's top, both included: what `self..` holds
    ## (D-526).
    fn count_to_top(self) -> FullCount

    ## A `..` range's count as a `..=` range's.
    fn widen_count(n: SpanCount) -> FullCount

    ## A `..=` range's count as a `..` range's, which holds it.
    fn narrow_count(n: FullCount) -> SpanCount

extend i8 implements Integer:
    type SpanCount = int
    type FullCount = int
    type Position = int

    fn widen_count(n: int) -> int:
        return n

    fn narrow_count(n: int) -> int:
        return n

    fn successor(self) -> i8:
        return self + 1

    fn predecessor(self) -> i8:
        return self - 1

    fn distance_to(self, end: i8) -> int:
        return (end as int) - (self as int)

    fn back_by(self, n: int) -> i8:
        return ((self as int) - n) as i8

    fn forward_by(self, n: int) -> i8:
        return ((self as int) + n) as i8

    fn count_to_top(self) -> int:
        return self.distance_to(i8.MAX) + 1

extend i16 implements Integer:
    type SpanCount = int
    type FullCount = int
    type Position = int

    fn widen_count(n: int) -> int:
        return n

    fn narrow_count(n: int) -> int:
        return n

    fn successor(self) -> i16:
        return self + 1

    fn predecessor(self) -> i16:
        return self - 1

    fn distance_to(self, end: i16) -> int:
        return (end as int) - (self as int)

    fn back_by(self, n: int) -> i16:
        return ((self as int) - n) as i16

    fn forward_by(self, n: int) -> i16:
        return ((self as int) + n) as i16

    fn count_to_top(self) -> int:
        return self.distance_to(i16.MAX) + 1

extend i32 implements Integer:
    type SpanCount = int
    type FullCount = int
    type Position = int

    fn widen_count(n: int) -> int:
        return n

    fn narrow_count(n: int) -> int:
        return n

    fn successor(self) -> i32:
        return self + 1

    fn predecessor(self) -> i32:
        return self - 1

    fn distance_to(self, end: i32) -> int:
        return (end as int) - (self as int)

    fn back_by(self, n: int) -> i32:
        return ((self as int) - n) as i32

    fn forward_by(self, n: int) -> i32:
        return ((self as int) + n) as i32

    fn count_to_top(self) -> int:
        return self.distance_to(i32.MAX) + 1

extend i64 implements Integer:
    type SpanCount = u64
    type FullCount = u128
    type Position = i128

    fn widen_count(n: u64) -> u128:
        return n

    fn narrow_count(n: u128) -> u64:
        return n as u64

    fn successor(self) -> i64:
        return self + 1

    fn predecessor(self) -> i64:
        return self - 1

    ## G8-4 — `end - self` in 64 unsigned bits wraps to the gap exactly, which is at most
    ## `u64.MAX`.
    @overflow(wrap)
    fn distance_to(self, end: i64) -> u64:
        return (end as u64) - (self as u64)

    @overflow(wrap)
    fn back_by(self, n: u64) -> i64:
        return ((self as u64) - n) as i64

    @overflow(wrap)
    fn forward_by(self, n: u64) -> i64:
        return ((self as u64) + n) as i64

    fn count_to_top(self) -> u128:
        return (self.distance_to(i64.MAX) as u128) + 1

extend i128 implements Integer:
    type SpanCount = u128
    type FullCount = u256
    type Position = i256

    fn widen_count(n: u128) -> u256:
        return n

    fn narrow_count(n: u256) -> u128:
        return n as u128

    fn successor(self) -> i128:
        return self + 1

    fn predecessor(self) -> i128:
        return self - 1

    ## G8-4 — `end - self` in 128 unsigned bits wraps to the gap exactly.
    @overflow(wrap)
    fn distance_to(self, end: i128) -> u128:
        return (end as u128) - (self as u128)

    @overflow(wrap)
    fn back_by(self, n: u128) -> i128:
        return ((self as u128) - n) as i128

    @overflow(wrap)
    fn forward_by(self, n: u128) -> i128:
        return ((self as u128) + n) as i128

    fn count_to_top(self) -> u256:
        return (self.distance_to(i128.MAX) as u256) + 1

extend isize implements Integer:
    type SpanCount = u64
    type FullCount = u128
    type Position = i128

    fn widen_count(n: u64) -> u128:
        return n

    fn narrow_count(n: u128) -> u64:
        return n as u64

    fn successor(self) -> isize:
        return self + 1

    fn predecessor(self) -> isize:
        return self - 1

    ## G8-4 — `end - self` in 64 unsigned bits wraps to the gap exactly, which is at most
    ## `u64.MAX`.
    @overflow(wrap)
    fn distance_to(self, end: isize) -> u64:
        return (end as u64) - (self as u64)

    @overflow(wrap)
    fn back_by(self, n: u64) -> isize:
        return ((self as u64) - n) as isize

    @overflow(wrap)
    fn forward_by(self, n: u64) -> isize:
        return ((self as u64) + n) as isize

    fn count_to_top(self) -> u128:
        return (self.distance_to(isize.MAX) as u128) + 1

extend u8 implements Integer:
    type SpanCount = int
    type FullCount = int
    type Position = int

    fn widen_count(n: int) -> int:
        return n

    fn narrow_count(n: int) -> int:
        return n

    fn successor(self) -> u8:
        return self + 1

    fn predecessor(self) -> u8:
        return self - 1

    fn distance_to(self, end: u8) -> int:
        return (end as int) - (self as int)

    fn back_by(self, n: int) -> u8:
        return ((self as int) - n) as u8

    fn forward_by(self, n: int) -> u8:
        return ((self as int) + n) as u8

    fn count_to_top(self) -> int:
        return self.distance_to(u8.MAX) + 1

extend u16 implements Integer:
    type SpanCount = int
    type FullCount = int
    type Position = int

    fn widen_count(n: int) -> int:
        return n

    fn narrow_count(n: int) -> int:
        return n

    fn successor(self) -> u16:
        return self + 1

    fn predecessor(self) -> u16:
        return self - 1

    fn distance_to(self, end: u16) -> int:
        return (end as int) - (self as int)

    fn back_by(self, n: int) -> u16:
        return ((self as int) - n) as u16

    fn forward_by(self, n: int) -> u16:
        return ((self as int) + n) as u16

    fn count_to_top(self) -> int:
        return self.distance_to(u16.MAX) + 1

extend u32 implements Integer:
    type SpanCount = int
    type FullCount = int
    type Position = int

    fn widen_count(n: int) -> int:
        return n

    fn narrow_count(n: int) -> int:
        return n

    fn successor(self) -> u32:
        return self + 1

    fn predecessor(self) -> u32:
        return self - 1

    fn distance_to(self, end: u32) -> int:
        return (end as int) - (self as int)

    fn back_by(self, n: int) -> u32:
        return ((self as int) - n) as u32

    fn forward_by(self, n: int) -> u32:
        return ((self as int) + n) as u32

    fn count_to_top(self) -> int:
        return self.distance_to(u32.MAX) + 1

extend u64 implements Integer:
    type SpanCount = u64
    type FullCount = u128
    type Position = i128

    fn widen_count(n: u64) -> u128:
        return n

    fn narrow_count(n: u128) -> u64:
        return n as u64

    fn successor(self) -> u64:
        return self + 1

    fn predecessor(self) -> u64:
        return self - 1

    fn distance_to(self, end: u64) -> u64:
        return (end - self)

    fn back_by(self, n: u64) -> u64:
        return self - n

    fn forward_by(self, n: u64) -> u64:
        return self + n

    fn count_to_top(self) -> u128:
        return (self.distance_to(u64.MAX) as u128) + 1

extend u128 implements Integer:
    type SpanCount = u128
    type FullCount = u256
    type Position = i256

    fn widen_count(n: u128) -> u256:
        return n

    fn narrow_count(n: u256) -> u128:
        return n as u128

    fn successor(self) -> u128:
        return self + 1

    fn predecessor(self) -> u128:
        return self - 1

    fn distance_to(self, end: u128) -> u128:
        return (end - self)

    fn back_by(self, n: u128) -> u128:
        return self - n

    fn forward_by(self, n: u128) -> u128:
        return self + n

    fn count_to_top(self) -> u256:
        return (self.distance_to(u128.MAX) as u256) + 1

extend usize implements Integer:
    type SpanCount = u64
    type FullCount = u128
    type Position = i128

    fn widen_count(n: u64) -> u128:
        return n

    fn narrow_count(n: u128) -> u64:
        return n as u64

    fn successor(self) -> usize:
        return self + 1

    fn predecessor(self) -> usize:
        return self - 1

    fn distance_to(self, end: usize) -> u64:
        return (end - self) as u64

    fn back_by(self, n: u64) -> usize:
        return self - (n as usize)

    fn forward_by(self, n: u64) -> usize:
        return self + (n as usize)

    fn count_to_top(self) -> u128:
        return (self.distance_to(usize.MAX) as u128) + 1

## `[STD-4]` — an integer that is not zero, made by `NonZero.new`. Since no
## `NonZero` holds 0, `Option[NonZero[T]]` stores `None` as 0 and is the size
## of `T` (`[TYP-13]`), and dividing by one needs no check for zero
## (`[EFF-16]`): `x // d` and `x % d` take a `NonZero` of `x`'s type.
@derive(Copy, Hash)
pub struct NonZero[T: Integer]:
    value: T

    ## `v`, or `None` when it is 0.
    pub fn new(v: T) -> Option[NonZero[T]]:
        if v == T.default():
            return None
        return Some(NonZero(v))

    ## The integer.
    pub fn get(self) -> T:
        return self.value

extend i8 implements FloorDiv[NonZero[i8]], Rem[NonZero[i8]]:
    type Output = i8

    fn floordiv(self, d: NonZero[i8]) -> i8:
        return self // d.value

    fn rem(self, d: NonZero[i8]) -> i8:
        return self % d.value

extend i16 implements FloorDiv[NonZero[i16]], Rem[NonZero[i16]]:
    type Output = i16

    fn floordiv(self, d: NonZero[i16]) -> i16:
        return self // d.value

    fn rem(self, d: NonZero[i16]) -> i16:
        return self % d.value

extend i32 implements FloorDiv[NonZero[i32]], Rem[NonZero[i32]]:
    type Output = i32

    fn floordiv(self, d: NonZero[i32]) -> i32:
        return self // d.value

    fn rem(self, d: NonZero[i32]) -> i32:
        return self % d.value

extend i64 implements FloorDiv[NonZero[i64]], Rem[NonZero[i64]]:
    type Output = i64

    fn floordiv(self, d: NonZero[i64]) -> i64:
        return self // d.value

    fn rem(self, d: NonZero[i64]) -> i64:
        return self % d.value

extend i128 implements FloorDiv[NonZero[i128]], Rem[NonZero[i128]]:
    type Output = i128

    fn floordiv(self, d: NonZero[i128]) -> i128:
        return self // d.value

    fn rem(self, d: NonZero[i128]) -> i128:
        return self % d.value

extend isize implements FloorDiv[NonZero[isize]], Rem[NonZero[isize]]:
    type Output = isize

    fn floordiv(self, d: NonZero[isize]) -> isize:
        return self // d.value

    fn rem(self, d: NonZero[isize]) -> isize:
        return self % d.value

extend u8 implements FloorDiv[NonZero[u8]], Rem[NonZero[u8]]:
    type Output = u8

    fn floordiv(self, d: NonZero[u8]) -> u8:
        return self // d.value

    fn rem(self, d: NonZero[u8]) -> u8:
        return self % d.value

extend u16 implements FloorDiv[NonZero[u16]], Rem[NonZero[u16]]:
    type Output = u16

    fn floordiv(self, d: NonZero[u16]) -> u16:
        return self // d.value

    fn rem(self, d: NonZero[u16]) -> u16:
        return self % d.value

extend u32 implements FloorDiv[NonZero[u32]], Rem[NonZero[u32]]:
    type Output = u32

    fn floordiv(self, d: NonZero[u32]) -> u32:
        return self // d.value

    fn rem(self, d: NonZero[u32]) -> u32:
        return self % d.value

extend u64 implements FloorDiv[NonZero[u64]], Rem[NonZero[u64]]:
    type Output = u64

    fn floordiv(self, d: NonZero[u64]) -> u64:
        return self // d.value

    fn rem(self, d: NonZero[u64]) -> u64:
        return self % d.value

extend u128 implements FloorDiv[NonZero[u128]], Rem[NonZero[u128]]:
    type Output = u128

    fn floordiv(self, d: NonZero[u128]) -> u128:
        return self // d.value

    fn rem(self, d: NonZero[u128]) -> u128:
        return self % d.value

extend usize implements FloorDiv[NonZero[usize]], Rem[NonZero[usize]]:
    type Output = usize

    fn floordiv(self, d: NonZero[usize]) -> usize:
        return self // d.value

    fn rem(self, d: NonZero[usize]) -> usize:
        return self % d.value
