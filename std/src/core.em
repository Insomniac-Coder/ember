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

extend String implements Default:
    fn default() -> String:
        return String.from("")

extend[T] Array[T] implements Default:
    fn default() -> Array[T]:
        return []

extend[T] Option[T] implements Default:
    fn default() -> Option[T]:
        return None

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

## The integer types, which `NonZero` takes. The interface is private, so no
## other type can join them.
interface Integer: Hash + Default:
    pass

extend i8 implements Integer:
    pass

extend i16 implements Integer:
    pass

extend i32 implements Integer:
    pass

extend i64 implements Integer:
    pass

extend i128 implements Integer:
    pass

extend isize implements Integer:
    pass

extend u8 implements Integer:
    pass

extend u16 implements Integer:
    pass

extend u32 implements Integer:
    pass

extend u64 implements Integer:
    pass

extend u128 implements Integer:
    pass

extend usize implements Integer:
    pass

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
