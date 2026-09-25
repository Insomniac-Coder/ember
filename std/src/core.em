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

## Part IV §8 — a receiver-less interface member is an associated function,
## invoked through the implementing type as `T.default()`.
pub interface Default:
    fn default() -> Self

pub interface Clone:
    fn clone(self) -> Self

## Part IV §8's canonical associated-type iterator contract. Named standard
## iterators, including the Arena-backed collection and Span iterators,
## implement this interface rather than introducing a second iterator
## abstraction.
pub interface Iterator:
    type Item
    fn next(mut self) -> Option[Item]

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

## Part IV §8 also declares `Hash`, `Display`, `Debug`, and the operator
## interfaces; they remain staged with their dependent surface.

## `[ERR-4]` (ODR-025) — the `Option` and `Result` methods that take a
## function. `x.map(f)` is a call of `option_map(x, f)`: the compiler routes the
## method here, so each is an ordinary generic function whose callback is
## checked, inferred and borrow-checked like any other. Each consumes the
## receiver and calls `f` at most once, before it returns; a payload passed to
## `f` is moved in (`owned`), except `filter`'s, which it must give back.

fn option_map[T, U](owned o: Option[T], f: fn(owned T) -> U) -> Option[U]:
    match o:
        Some(v):
            return Some(f(v))
        None:
            return None

fn option_and_then[T, U](owned o: Option[T], f: fn(owned T) -> Option[U]) -> Option[U]:
    match o:
        Some(v):
            return f(v)
        None:
            return None

fn option_filter[T](owned o: Option[T], f: fn(T) -> bool) -> Option[T]:
    match o:
        Some(v):
            if f(v):
                return Some(v)
            return None
        None:
            return None

fn option_or_else[T](owned o: Option[T], f: fn() -> Option[T]) -> Option[T]:
    match o:
        Some(v):
            return Some(v)
        None:
            return f()

fn option_unwrap_or_else[T](owned o: Option[T], f: fn() -> T) -> T:
    match o:
        Some(v):
            return v
        None:
            return f()

fn option_ok_or_else[T, E](owned o: Option[T], f: fn() -> E) -> Result[T, E]:
    match o:
        Some(v):
            return Ok(v)
        None:
            return Err(f())

fn result_map[T, E, U](owned r: Result[T, E], f: fn(owned T) -> U) -> Result[U, E]:
    match r:
        Ok(v):
            return Ok(f(v))
        Err(e):
            return Err(e)

fn result_map_err[T, E, F](owned r: Result[T, E], f: fn(owned E) -> F) -> Result[T, F]:
    match r:
        Ok(v):
            return Ok(v)
        Err(e):
            return Err(f(e))

fn result_and_then[T, E, U](owned r: Result[T, E], f: fn(owned T) -> Result[U, E]) -> Result[U, E]:
    match r:
        Ok(v):
            return f(v)
        Err(e):
            return Err(e)

fn result_or_else[T, E, F](owned r: Result[T, E], f: fn(owned E) -> Result[T, F]) -> Result[T, F]:
    match r:
        Ok(v):
            return Ok(v)
        Err(e):
            return f(e)

fn result_unwrap_or_else[T, E](owned r: Result[T, E], f: fn(owned E) -> T) -> T:
    match r:
        Ok(v):
            return v
        Err(e):
            return f(e)

## `[STD-15]` — the `Array` methods written in Ember (`[GRM-34]`). A method
## the compiler knows by a name comes first (`[TYP-24]`); these are found
## after it, and one a program never calls is not emitted (`[COST-1]`).
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
