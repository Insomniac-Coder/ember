## `std.core` — Part XV's prelude module.
##
## Part IV §8's standard interfaces, spelled as ordinary interfaces exactly as
## that section writes them. `Option`, `Result`, `Array` and `String` are still
## compiler-known (Part XX.1 makes them so "until Phase 2's generics let the
## standard library write them"); this module holds what can already be
## written, and grows as the compiler does.

import std.mem
from std.fmt import Formatter, FmtError

## `[ENM-3]` — a unit-only enum, so `Copy`, `Eq`, `Hash` and `Debug` come free
## and it converts to its repr with `as`.
pub enum Ordering:
    Less
    Equal
    Greater

pub interface Eq:
    fn eq(other: Self) -> bool

## Part IV §8 declares `Ord: Eq` with `cmp(other: Self) -> Ordering`.
## `[TYP-9]` — `Ord` is deliberately **not** implemented for floats: `NaN`
## makes the ordering partial, so a float uses `PartialOrd` and the comparison
## operators, which return `false` on either side of a `NaN`.
pub interface Ord: needs Eq:
    fn cmp(other: Self) -> Ordering

## Part IV §8 — a receiver-less interface member is an associated function,
## invoked through the implementing type as `T.default()`.
pub interface Default:
    static fn default() -> Self

## `[DRV-1]` — `T`'s `Default`, as `@derive(Default)` gives a field declared without one.
[T: needs Default]
pub fn default_of() -> T:
    return T.default()

pub interface Clone:
    fn clone() -> Self

## Part IV §8's marker interfaces: no methods. The compiler says which types have them, from what
## they hold (derived automatically: `[THR-8]`, `@derive(Copy)`); a program writes `Send` or `Sync`
## by hand only with `unsafe extend` (ODR-111), and `Copy` and `Sized` never.
pub interface Copy:
    pass

pub interface Send:
    pass

pub interface Sync:
    pass

pub interface Sized:
    pass

## Part IV §8 — a value's text, as `f"{x}"`, `print(x)` and `main`'s error
## write it. The numbers, `bool`, `char`, text, the collections, tuples,
## `Option`, `Result`, `Box` and `Cell` have it from the table of
## `[TYP-36]`, which the compiler writes; a struct, an enum or a class has it
## only by implementing it, and writes its text through `f` (`std.fmt`).
pub interface Display:
    fn fmt(ref mut f: Formatter) -> Result[void, FmtError]

## Part IV §8 — a `T` made into the implementing type: `F.from(v)`. `e?`
## converts a failure into the function's error type with it (`[ERR-2]`).
pub interface From[T]:
    static fn from(owns value: T) -> Self

## Part IV §8 — a value's structure, as `f"{x!r}"` writes it and as it shows
## inside another value's text (an element, a field, a payload). Every type
## the table gives text has it, and a struct or an enum field-wise without
## asking (`[STR-5]`); one it writes replaces that one.
pub interface Debug:
    fn fmt_debug(ref mut f: Formatter) -> Result[void, FmtError]

## `[ERR-3]` — an error type: its text (`Display`), its structure (`Debug`),
## and the error that caused it, if one did. `@derive(Error)` writes `Display`
## from `@error("…")` attributes and `source` from an `@from` variant.
pub interface Error: needs (Debug, Display):
    fn source() -> Option[ref dyn Error]:
        return None

## `[ERR-8]` — any error: one error of any type implementing `Error`, boxed,
## with the context messages `context` and `with_context` added to it
## (`[ERR-10]`), the last added outermost. `?` converts any error into one
## with its `From`, allocating once, on the failure path (`[ERR-11]`). It is
## not an `Error` itself: its `From[E]` is for every `E: Error`, and every
## type's own `From` (`[ERR-7]`) moves an `AnyError` on through `?`.
## `identity` is the boxed error's type (`type_identity`), for `is` and
## `downcast_ref`.
pub struct AnyError:
    error: Box[dyn Error]
    identity: u128
    context: Array[String]

extend[E: needs Error] AnyError implements From[E]:
    static fn from(owns value: E) -> AnyError:
        return AnyError(Box(value), type_identity[E](), [])

## `[ERR-10]` — its text is each context message, the last added first, then
## the error's own: `reading app.cfg: line 3: expected `key = value``.
extend AnyError implements Display:
    ## It allocates: it writes into the formatter's text, and the boxed error's own `fmt` is a call through `dyn Error` (`[STD-1]`).
    fn fmt(ref mut f: Formatter) -> Result[void, FmtError]:
        at = this.context.len()
        while at > 0:
            at -= 1
            f.write_str(this.context[at])?
            f.write_str(": ")?
        return this.error.fmt(ref mut f)

## `AnyError(<the error's Debug>)`, and the context messages when there are
## any, oldest first.
extend AnyError implements Debug:
    ## It allocates: it writes into the formatter's text (`[STD-1]`).
    fn fmt_debug(ref mut f: Formatter) -> Result[void, FmtError]:
        f.write_str("AnyError(")?
        this.error.fmt_debug(ref mut f)?
        if this.context.len() > 0:
            f.write_str(f", context={this.context}")?
        return f.write_str(")")

extend AnyError:
    ## The error's cause, past the text this one shows: the boxed error's
    ## `source` (`[ERR-12]` prints one `caused by:` line for each).
    ## It allocates when the boxed error's `source` does: a call through `dyn Error`, whose effects nothing declares (`[STD-1]`).
    pub fn source() -> Option[ref dyn Error]:
        return this.error.source()

    ## Whether the boxed error is an `E`.
    [E]
    pub fn is() -> bool:
        return this.identity == type_identity[E]()

    ## The boxed error, as the `E` it is, or `None` when it is another type.
    [E]
    pub fn downcast_ref() -> Option[ref E]:
        if this.identity != type_identity[E]():
            return None
        unsafe:  # SAFETY: `identity` is the boxed value's type's, so the box holds an `E`
            return Some(dyn_payload[E](this.error))

## `[ERR-12]` — how `main` reports an `Err` that is an `Error`: its text,
## then one `caused by:` line for each cause in its chain.
[E: needs Error]
fn report_error(e: E):
    eprintln(f"error: {e}")
    report_causes(e.source())

## `[ERR-12]` — the same for an `AnyError`, whose text holds its context
## messages and whose causes are its error's (`AnyError.source`).
## It allocates: it formats the error's text (`[STD-1]`).
fn report_any_error(e: AnyError):
    eprintln(f"error: {e}")
    report_causes(e.source())

## It allocates: it formats each cause's text (`[STD-1]`).
fn report_causes(first: Option[ref dyn Error]):
    cause = first
    while true:
        match cause:
            Some(c):
                eprintln(f"caused by: {c}")
                cause = c.source()
            None:
                return

## `[ERR-10]` — `r.context(msg)`: the error, if any, with `msg`'s text added
## outermost, so it displays as `msg: <what it displayed>`. The compiler
## converts `r`'s error into an `AnyError` first (`[ERR-8]`); the text is
## made only on failure.
[T, M: needs Display]
fn result_context(owns r: Result[T, AnyError], msg: M) -> Result[T, AnyError]:
    match owns r:
        Ok(v):
            return Ok(v)
        Err(e):
            e.context.push(owns f"{msg}")
            return Err(e)

## `[ERR-10]` — `r.with_context(f)`: as `context`, with the message `f`
## makes, called only on failure.
[T, M: needs Display, Call: needs once fn() -> M]
fn result_with_context(owns r: Result[T, AnyError], owns f: Call) -> Result[T, AnyError]:
    match owns r:
        Ok(v):
            return Ok(v)
        Err(e):
            e.context.push(owns f"{f()}")
            return Err(e)

## `[ERR-4]` — `o.context(msg)` on an `Option`: `Ok` of its value, or for
## `None` an `AnyError` whose text is `msg`'s (`Missing`); the text is made
## only for `None`.
[T, M: needs Display]
fn option_context(owns o: Option[T], msg: M) -> Result[T, AnyError]:
    match owns o:
        Some(v):
            return Ok(v)
        None:
            return Err(AnyError.from(owns Missing(f"{msg}")))

## `[ERR-4]` — `o.with_context(f)`: as `context`, with the message `f`
## makes, called only for `None`.
[T, M: needs Display, Call: needs once fn() -> M]
fn option_with_context(owns o: Option[T], owns f: Call) -> Result[T, AnyError]:
    match owns o:
        Some(v):
            return Ok(v)
        None:
            return Err(AnyError.from(owns Missing(f"{f()}")))

## ODR-180 — a failure that is a message: what Python's `raise ValueError("…")` means, written
## `return Err(AnyError.msg("…"))` in a function returning `Result[T]`. Its text is the message.
extend AnyError:
    ## It allocates: the error is boxed (`[ERR-11]`), on the failure path only.
    pub static fn msg(owns text: String) -> AnyError:
        return AnyError(Box(Message(text)), type_identity[Message](), [])

## The error `AnyError.msg` makes: its text alone.
@derive(Error)
@error("{text}")
struct Message:
    text: String

## The error an `Option`'s `context` makes of a `None`: its text alone.
@derive(Error)
@error("{text}")
struct Missing:
    text: String

## `[CORO-4]` — a generator's suspension points, compiler-known. A `gen fn`'s
## body is checked as a function that calls `__coroutine_start()` first and
## `__coroutine_yield[Y](e)` for each `yield e`; the coroutine transform turns
## each call into a suspension of the state machine (ADR-193), and none runs.
## The answer is whether the generator was resumed rather than dropped.
pub fn __coroutine_start() -> bool:
    return true

[T]
pub fn __coroutine_yield(owns _value: T) -> bool:
    return true

## A `yield` that is a `=>` arm's or a conditional's branch, as the `void` it is.
pub fn __coroutine_resumed(_resumed: bool):
    pass

## `[ERR-8]` — a type's identity: a 128-bit hash of its canonical name, the
## same for one type in every module and every build. Compiler-known.
[T]
fn type_identity() -> u128:
    pass

## `[ERR-8]` — the boxed `dyn Error`, borrowed, as the `ref E` it was made
## from. Compiler-known; the box must hold an `E`.
@safety("the caller has checked the boxed value is an `E`")
[E]
unsafe fn dyn_payload(boxed: Box[dyn Error]) -> ref E:
    pass

## `[ERR-7]` — every type converts from itself, unchanged, so `?` passes on
## an error of the function's own error type as it is (a move: `[ERR-2]`
## makes no call when the two types are one). An extension over a type
## parameter standing alone is every type's (ADR-186); a program's own
## `extend T implements From[T]` would be a second one (`E2041`).
extend[T] T implements From[T]:
    static fn from(owns value: T) -> T:
        return value

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
    fn add(rhs: Rhs) -> Output

pub interface Sub[Rhs = Self]:
    type Output
    fn sub(rhs: Rhs) -> Output

pub interface Mul[Rhs = Self]:
    type Output
    fn mul(rhs: Rhs) -> Output

pub interface Div[Rhs = Self]:
    type Output
    fn div(rhs: Rhs) -> Output

pub interface FloorDiv[Rhs = Self]:
    type Output
    fn floordiv(rhs: Rhs) -> Output

pub interface Rem[Rhs = Self]:
    type Output
    fn rem(rhs: Rhs) -> Output

pub interface Pow[Rhs = Self]:
    type Output
    fn pow(rhs: Rhs) -> Output

pub interface BitAnd[Rhs = Self]:
    type Output
    fn bitand(rhs: Rhs) -> Output

pub interface BitOr[Rhs = Self]:
    type Output
    fn bitor(rhs: Rhs) -> Output

pub interface BitXor[Rhs = Self]:
    type Output
    fn bitxor(rhs: Rhs) -> Output

pub interface Shl[Rhs = Self]:
    type Output
    fn shl(rhs: Rhs) -> Output

pub interface Shr[Rhs = Self]:
    type Output
    fn shr(rhs: Rhs) -> Output

## Unary `-` and `~`.
pub interface Neg:
    type Output
    fn neg() -> Output

pub interface Not:
    type Output
    fn not() -> Output

pub interface AddAssign[Rhs = Self]:
    fn add_assign(rhs: Rhs)

pub interface SubAssign[Rhs = Self]:
    fn sub_assign(rhs: Rhs)

pub interface MulAssign[Rhs = Self]:
    fn mul_assign(rhs: Rhs)

pub interface DivAssign[Rhs = Self]:
    fn div_assign(rhs: Rhs)

pub interface FloorDivAssign[Rhs = Self]:
    fn floordiv_assign(rhs: Rhs)

pub interface RemAssign[Rhs = Self]:
    fn rem_assign(rhs: Rhs)

pub interface PowAssign[Rhs = Self]:
    fn pow_assign(rhs: Rhs)

pub interface BitAndAssign[Rhs = Self]:
    fn bitand_assign(rhs: Rhs)

pub interface BitOrAssign[Rhs = Self]:
    fn bitor_assign(rhs: Rhs)

pub interface BitXorAssign[Rhs = Self]:
    fn bitxor_assign(rhs: Rhs)

pub interface ShlAssign[Rhs = Self]:
    fn shl_assign(rhs: Rhs)

pub interface ShrAssign[Rhs = Self]:
    fn shr_assign(rhs: Rhs)

## `a[i]` reads through `Index.index`, and is written in place through
## `IndexMut.index_mut`; `a[i] = v` calls `IndexSet.index_set` where the type
## has it, which is how a `Map` inserts (`[STD-17]`).
pub interface Index[Idx]:
    type Output
    fn index(i: Idx) -> ref Output

pub interface IndexMut[Idx]: needs Index[Idx]:
    fn index_mut(i: Idx) -> ref mut Output

pub interface IndexSet[Idx, V]:
    fn index_set(i: Idx, owns v: V)

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
pub interface ItemCount: needs (Ord, Copy):
    ## The count of `n` items, which is not negative.
    static fn of(n: int) -> Self
    ## The count `n`, which fits: how counts of different types meet.
    static fn of_wide(n: u256) -> Self
    ## This count as a `u256`, which holds every count; a negative number has none.
    fn to_wide() -> u256

    ## The count `n`, or this type's largest where `n` is past it.
    static fn at_most(n: u256) -> Self

    ## This count in the count type `C`, which holds it.
    [C: needs ItemCount]
    fn as_count() -> C:
        return C.of_wide(this.to_wide())
    ## Whether this is the type's largest value, which has no next one.
    fn at_top() -> bool
    ## This count and `other`'s items together.
    fn plus(other: Self) -> Self
    ## This count less `other`, which is not more than it.
    fn less(other: Self) -> Self
    ## Whether this count is more than `n` items.
    fn exceeds(n: int) -> bool
    ## This count divided by `k`, which is positive, rounded down.
    fn div_by(k: int) -> Self
    ## What is left of this count after dividing it by `k`, which is positive.
    fn rem_by(k: int) -> int
    ## This count as an `int`; one past `int.MAX` panics.
    fn to_int() -> int

extend i64 implements ItemCount:
    static fn of(n: int) -> i64:
        return n

    static fn at_most(n: u256) -> i64:
        if n > (int.MAX as u256):
            return int.MAX
        return n as i64

    ## It allocates the text of its panic message when it panics (an f-string, `[PAN-2]`).
    static fn of_wide(n: u256) -> i64:
        if n > (int.MAX as u256):
            panic(f"{n} does not fit an int")
        return n as i64

    ## It allocates the text of its panic message when it panics (an f-string, `[PAN-2]`).
    fn to_wide() -> u256:
        if this < 0:
            panic(f"{this} is not a count")
        return this as u256

    fn at_top() -> bool:
        return this == 9223372036854775807

    fn plus(other: i64) -> i64:
        return this + other

    fn less(other: i64) -> i64:
        return this - other

    fn exceeds(n: int) -> bool:
        return this > n

    fn div_by(k: int) -> i64:
        return this // k

    fn rem_by(k: int) -> int:
        return this % k

    fn to_int() -> int:
        return this

## G8-4 — what a `..` range of 64-bit numbers counts in.
extend u64 implements ItemCount:
    ## It allocates the text of its panic message when it panics (an f-string, `[PAN-2]`).
    static fn of(n: int) -> u64:
        if n < 0:
            panic(f"a count cannot be negative: {n}")
        return n as u64

    static fn at_most(n: u256) -> u64:
        if n > (u64.MAX as u256):
            return u64.MAX
        return n as u64

    ## It allocates the text of its panic message when it panics (an f-string, `[PAN-2]`).
    static fn of_wide(n: u256) -> u64:
        if n > (u64.MAX as u256):
            panic(f"{n} does not fit a u64")
        return n as u64

    fn to_wide() -> u256:
        return this

    fn at_top() -> bool:
        return this == 18446744073709551615

    fn plus(other: u64) -> u64:
        return this + other

    fn less(other: u64) -> u64:
        return this - other

    fn exceeds(n: int) -> bool:
        return n < 0 or this > (n as u64)

    fn div_by(k: int) -> u64:
        return this // (k as u64)

    fn rem_by(k: int) -> int:
        return (this % (k as u64)) as int

    ## It allocates the text of its panic message when it panics (an f-string, `[PAN-2]`).
    fn to_int() -> int:
        if this > 9223372036854775807:
            panic(f"a count of {this} does not fit an int")
        return this as int

## G8-4 — what a `..` range of 128-bit numbers counts in, and a `..=` one of 64-bit numbers.
extend u128 implements ItemCount:
    ## It allocates the text of its panic message when it panics (an f-string, `[PAN-2]`).
    static fn of(n: int) -> u128:
        if n < 0:
            panic(f"a count cannot be negative: {n}")
        return n as u128

    static fn at_most(n: u256) -> u128:
        if n > (u128.MAX as u256):
            return u128.MAX
        return n as u128

    ## It allocates the text of its panic message when it panics (an f-string, `[PAN-2]`).
    static fn of_wide(n: u256) -> u128:
        if n > (u128.MAX as u256):
            panic(f"{n} does not fit a u128")
        return n as u128

    fn to_wide() -> u256:
        return this

    fn at_top() -> bool:
        return this == u128.MAX

    fn plus(other: u128) -> u128:
        return this + other

    fn less(other: u128) -> u128:
        return this - other

    fn exceeds(n: int) -> bool:
        return n < 0 or this > (n as u128)

    fn div_by(k: int) -> u128:
        return this // (k as u128)

    fn rem_by(k: int) -> int:
        return (this % (k as u128)) as int

    ## It allocates the text of its panic message when it panics (an f-string, `[PAN-2]`).
    fn to_int() -> int:
        if this > 9223372036854775807:
            panic(f"a count of {this} does not fit an int")
        return this as int

## G8-4 — what `enumerate` numbers a range of 64-bit numbers with.
extend i128 implements ItemCount:
    static fn of(n: int) -> i128:
        return n

    static fn at_most(n: u256) -> i128:
        if n > (i128.MAX as u256):
            return i128.MAX
        return n as i128

    ## It allocates the text of its panic message when it panics (an f-string, `[PAN-2]`).
    static fn of_wide(n: u256) -> i128:
        if n > (i128.MAX as u256):
            panic(f"{n} does not fit an i128")
        return n as i128

    ## It allocates the text of its panic message when it panics (an f-string, `[PAN-2]`).
    fn to_wide() -> u256:
        if this < 0:
            panic(f"{this} is not a count")
        return this as u256

    fn at_top() -> bool:
        return this == i128.MAX

    fn plus(other: i128) -> i128:
        return this + other

    fn less(other: i128) -> i128:
        return this - other

    fn exceeds(n: int) -> bool:
        return this > (n as i128)

    fn div_by(k: int) -> i128:
        return this // (k as i128)

    fn rem_by(k: int) -> int:
        return (this % (k as i128)) as int

    ## It allocates the text of its panic message when it panics (an f-string, `[PAN-2]`).
    fn to_int() -> int:
        if this > (int.MAX as i128) or this < (int.MIN as i128):
            panic(f"a number of {this} does not fit an int")
        return this as int

## G8-4 — what `enumerate` numbers a range of 128-bit numbers with (`i256`, the signed 256-bit
## count: a start may be negative).
extend i256 implements ItemCount:
    static fn of(n: int) -> i256:
        return n

    ## A count past the largest `i256` has no number: it panics.
    ## It allocates the text of its panic message when it panics (an f-string, `[PAN-2]`).
    static fn at_most(n: u256) -> i256:
        return i256.of_wide(n)

    ## It allocates the text of its panic message when it panics (an f-string, `[PAN-2]`).
    static fn of_wide(n: u256) -> i256:
        number = n as i256
        if number < 0:
            panic(f"{n} does not fit an i256")
        return number

    ## It allocates the text of its panic message when it panics (an f-string, `[PAN-2]`).
    fn to_wide() -> u256:
        if this < 0:
            panic(f"{this} is not a count")
        return this as u256

    ## Never: no number comes near its top.
    fn at_top() -> bool:
        return false

    fn plus(other: i256) -> i256:
        return this + other

    fn less(other: i256) -> i256:
        return this - other

    fn exceeds(n: int) -> bool:
        return this > (n as i256)

    ## It allocates the text of its panic message when it panics (an f-string, `[PAN-2]`).
    fn div_by(k: int) -> i256:
        return count_divmod(this.to_wide(), k as u256).0 as i256

    ## It allocates the text of its panic message when it panics (an f-string, `[PAN-2]`).
    fn rem_by(k: int) -> int:
        return count_divmod(this.to_wide(), k as u256).1 as int

    ## It allocates the text of its panic message when it panics (an f-string, `[PAN-2]`).
    fn to_int() -> int:
        if this > (int.MAX as i256) or this < (int.MIN as i256):
            panic(f"a number of {this} does not fit an int")
        return this as int

## G8-4 — what a `..=` range of 128-bit numbers counts in (`u256`, the 256-bit count). It adds,
## subtracts and compares, so it divides by doubling.
extend u256 implements ItemCount:
    ## It allocates the text of its panic message when it panics (an f-string, `[PAN-2]`).
    static fn of(n: int) -> u256:
        if n < 0:
            panic(f"a count cannot be negative: {n}")
        return n as u256

    static fn at_most(n: u256) -> u256:
        return n

    static fn of_wide(n: u256) -> u256:
        return n

    fn to_wide() -> u256:
        return this

    ## Never: no count comes near its top.
    fn at_top() -> bool:
        return false

    fn plus(other: u256) -> u256:
        return this + other

    fn less(other: u256) -> u256:
        return this - other

    fn exceeds(n: int) -> bool:
        return n < 0 or this > (n as u256)

    fn div_by(k: int) -> u256:
        return count_divmod(this, k as u256).0

    fn rem_by(k: int) -> int:
        return count_divmod(this, k as u256).1 as int

    ## It allocates the text of its panic message when it panics (an f-string, `[PAN-2]`).
    fn to_int() -> int:
        if this > (int.MAX as u256):
            panic(f"a count of {this} does not fit an int")
        return this as int

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
    type Count: needs ItemCount = int
    ## And what `enumerate` numbers its items with.
    type Position: needs ItemCount = int
    fn next() -> Option[Item]

    ## `[STD-19]` (ODR-089) — the adapters. Each takes this iterator and
    ## gives a new one; nothing runs until the new one's `next` is called. A
    ## bad count panics here, where the bug is, never partway through a loop.

    ## At most `n` items.
    fn take(n: int) owns this -> Take[Self]:
        if n < 0:
            panic(f"take({n}): a count cannot be negative")
        return Take(this, n)

    ## All but the first `n` items.
    fn skip(n: int) owns this -> Skip[Self]:
        if n < 0:
            panic(f"skip({n}): a count cannot be negative")
        return Skip(this, n)

    ## The first item, then every `k`-th.
    fn step_by(k: int) owns this -> StepBy[Self]:
        if k <= 0:
            panic(f"step_by({k}): the step must be positive")
        return StepBy(this, k - 1, false)

    ## Each item with its number, counting from `start` (`[STD-26]`).
    fn enumerate(start: int = 0) owns this -> Enumerate[Self, Position]:
        return Enumerate(this, Position.of(start), false)

    ## Pairs of an item of each, until either runs out.
    [J: needs Iterator]
    fn zip(owns other: J) owns this -> Zip[Self, J]:
        return Zip(this, other)

    ## This iterator's items, then `other`'s, which are of the same type.
    [J: needs Iterator[Item = Item]]
    fn chain(owns other: J) owns this -> Chain[Self, J]:
        return Chain(this, other, false)

    ## What `f` makes of each item. The adapter holds `f`, so nothing is
    ## allocated and each call is direct (`[CLO-14]`). A named callable is
    ## lent to it (`ref mut`), the caller keeping it, and the iterator lives no
    ## longer than that loan; a lambda written at the call is given to it and
    ## keeps the captures `[CLO-2]` infers (`[CLO-15]`), so the iterator may be
    ## kept or returned as long as they live (`[CLO-4]`, `[STD-19]`, ODR-240).
    ## The other adapters do the same.
    [R, F: needs fn(Item) -> R]
    fn map(owns f: F) owns this -> Map[Self, R, F]:
        return Map(this, f)

    ## The items `pred` holds for.
    [F: needs fn(Item) -> bool]
    fn filter(owns pred: F) owns this -> Filter[Self, F]:
        return Filter(this, pred)

    ## What `f` gives for each item it gives something for.
    [R, F: needs fn(Item) -> Option[R]]
    fn filter_map(owns f: F) owns this -> FilterMap[Self, R, F]:
        return FilterMap(this, f)

    ## The items before the first one `pred` does not hold for.
    [F: needs fn(Item) -> bool]
    fn take_while(owns pred: F) owns this -> TakeWhile[Self, F]:
        return TakeWhile(this, pred, false)

    ## The items from the first one `pred` does not hold for.
    [F: needs fn(Item) -> bool]
    fn skip_while(owns pred: F) owns this -> SkipWhile[Self, F]:
        return SkipWhile(this, pred, false)

    ## The items, each given to `f` first.
    [F: needs fn(Item)]
    fn inspect(owns f: F) owns this -> Inspect[Self, F]:
        return Inspect(this, f)

    ## The items of each iterable `f` makes of an item, one after another.
    [U: needs IntoIterator, F: needs fn(Item) -> U]
    fn flat_map(owns f: F) owns this -> FlatMap[Self, U, F]:
        return FlatMap(this, f, None)

    ## `[GRM-38]` — the compiler's, for a generator expression's later `for` clause: each item
    ## with each item of the iterator `f` makes of it, as a pair. The item is copied into each
    ## pair, so it is `Copy` (a reference to an element, a number).
    [U: needs Iterator, F: needs fn(Item) -> U]
    fn __nested(owns f: F) owns this -> Nested[Self, Item, U, F] where Item: needs Copy:
        return Nested(this, f, None, None)

    ## The items of each item, one after another, when the items are
    ## iterable themselves.
    fn flatten() owns this -> Flatten[Self, Item] where Item: needs IntoIterator:
        return Flatten(this, None)

    ## These items, with `peek` to look at the next one without taking it
    ## (ODR-094).
    fn peekable() owns this -> Peekable[Self]:
        return Peekable(this, None, false)

    ## `[STD-19]` — the consumers. Each takes this iterator and runs it.

    ## How many items there are.
    fn count() owns this -> Count:
        n = Count.of(0)
        one = Count.of(1)
        for _ in this:
            n = n.plus(one)
        return n

    ## The last item, if any.
    fn last() owns this -> Option[Item]:
        found: Option[Item] = None
        for x in this:
            found = Some(x)
        return found

    ## The item at index `n`, if there are that many.
    fn nth(n: int) owns this -> Option[Item]:
        if n < 0:
            panic(f"nth({n}): an index cannot be negative")
        rest = this
        if rest.skip_front(Count.of(n)) > Count.of(0):
            return None
        return rest.next()

    ## `[STD-19]` (G8-4) — leaves out up to `n` items from the front, and gives how many of the `n`
    ## it could not (none when all went). A range does it at once, and the adapters that never
    ## change their items pass it down, so no gigantic run is passed one item at a time.
    fn skip_front(n: Count) -> Count:
        left = n
        zero = Count.of(0)
        one = Count.of(1)
        while left > zero:
            if this.next().is_none():
                return left
            left = left.less(one)
        return left

    ## `f(... f(f(init, a), b) ..., z)` over the items `a` to `z`. The consumers, as the adapters,
    ## borrow a named callable with leave to change what it captured (`ref mut`), and are given a
    ## lambda written at the call (`[STD-19]`, ODR-240), so it may write what it captures
    ## (`[CLO-2]`) and the caller reads that once the callable's last use is past.
    [B, F: needs fn(B, Item) -> B]
    fn fold(init: B, owns f: F) owns this -> B:
        acc = init
        for x in this:
            acc = f(acc, x)
        return acc

    ## Whether `pred` holds for some item; stops at the first.
    [F: needs fn(Item) -> bool]
    fn any(owns pred: F) owns this -> bool:
        for x in this:
            if pred(x):
                return true
        return false

    ## Whether `pred` holds for every item; stops at the first it does not.
    [F: needs fn(Item) -> bool]
    fn all(owns pred: F) owns this -> bool:
        for x in this:
            if not pred(x):
                return false
        return true

    ## The first item `pred` holds for. An item is the iterator's, never `pred`'s (`[LT-1a]`).
    [F: needs fn(Item) -> bool]
    fn find(owns pred: F) owns this -> Option[Item]:
        for x in this:
            if pred(x):
                return Some(x)
        return None

    ## The index of the first item `pred` holds for.
    [F: needs fn(Item) -> bool]
    fn position(owns pred: F) owns this -> Option[Count]:
        at = Count.of(0)
        one = Count.of(1)
        for x in this:
            if pred(x):
                return Some(at)
            at = at.plus(one)
        return None

    ## `f` on each item, in order. Lent or given as an adapter's callable is (`[STD-19]`,
    ## ODR-240), so it may write what it captures (`[CLO-2]`):
    ## `xs.iter().for_each(fn(x): total += x)`.
    [F: needs fn(Item)]
    fn for_each(owns f: F) owns this:
        for x in this:
            f(x)

    ## The largest item; the last of equal largest ones. `None` when empty.
    fn max() owns this -> Option[Item] where Item: needs Ord:
        best: Option[Item] = None
        for x in this:
            match best:
                Some(b):
                    if not (x < b):
                        best = Some(x)
                None:
                    best = Some(x)
        return best

    ## The smallest item; the first of equal smallest ones. `None` when empty.
    fn min() owns this -> Option[Item] where Item: needs Ord:
        best: Option[Item] = None
        for x in this:
            match best:
                Some(b):
                    if x < b:
                        best = Some(x)
                None:
                    best = Some(x)
        return best

    ## The item whose `key` is largest; the last of equal ones.
    [K: needs Ord, F: needs fn(Item) -> K]
    fn max_by_key(owns key: F) owns this -> Option[Item]:
        best: Option[Item] = None
        best_key: Option[K] = None
        for x in this:
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
    [K: needs Ord, F: needs fn(Item) -> K]
    fn min_by_key(owns key: F) owns this -> Option[Item]:
        best: Option[Item] = None
        best_key: Option[K] = None
        for x in this:
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
    [F: needs fn(Item, Item) -> Ordering]
    fn max_by(owns cmp: F) owns this -> Option[Item]:
        best: Option[Item] = None
        for x in this:
            match best:
                Some(b):
                    if cmp(x, b) != Ordering.Less:
                        best = Some(x)
                None:
                    best = Some(x)
        return best

    ## The smallest item by `cmp`; the first of equal ones.
    [F: needs fn(Item, Item) -> Ordering]
    fn min_by(owns cmp: F) owns this -> Option[Item]:
        best: Option[Item] = None
        for x in this:
            match best:
                Some(b):
                    if cmp(x, b) == Ordering.Less:
                        best = Some(x)
                None:
                    best = Some(x)
        return best

    ## `f(... f(f(a, b), c) ..., z)` over the items; `None` when empty.
    [F: needs fn(Item, Item) -> Item]
    fn reduce(owns f: F) owns this -> Option[Item]:
        acc: Option[Item] = None
        for x in this:
            match acc:
                Some(a):
                    acc = Some(f(a, x))
                None:
                    acc = Some(x)
        return acc

    ## The items, in order, in a new array.
    fn to_array() owns this -> Array[Item]:
        out: Array[Item] = []
        for x in this:
            out.push(owns x)
        return out

    ## The items gathered into a collection `C` (ODR-094): any type that can
    ## be built from them (`FromIterator`), written as `collect[C]()` or taken
    ## from the type the call is expected to give (`a: Array[int] =
    ## it.collect()`).
    [C: needs FromIterator[Item]]
    fn collect() owns this -> C:
        return C.from_iter(owns this)

    ## The items as text, `sep` between each two (`[1, 2, 3]` with `", "` is
    ## `"1, 2, 3"`).
    fn join(sep: StrView) owns this -> String where Item: needs Display:
        out = String()
        first = true
        for x in this:
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
pub interface DoubleEndedIterator: needs Iterator:
    fn next_back() -> Option[Item]

    ## `[STD-19]` (G8-4) — leaves out up to `n` items from the back, and gives how many of the `n`
    ## it could not (none when all went). A range does it at once, and the adapters pass it down.
    fn skip_back(n: Count) -> Count:
        left = n
        zero = Count.of(0)
        one = Count.of(1)
        while left > zero:
            if this.next_back().is_none():
                return left
            left = left.less(one)
        return left

    ## `[STD-19]` (G8-4) — leaves out `n` items from the back and gives the one before them:
    ## `nth_back(Count.of(0))` is `next_back()`.
    fn nth_back(n: Count) -> Option[Item]:
        if this.skip_back(n) > Count.of(0):
            return None
        return this.next_back()

    ## The items, last first.
    fn rev() owns this -> Rev[Self]:
        return Rev(this)

## `[STD-19]` (ODR-091) — an iterator that knows exactly how many items it has
## left (`len`), without giving them: the ranges of integers, the views'
## iterators, and the adapters over ones.
pub interface ExactSizeIterator: needs Iterator:
    fn len() -> Count

pub interface IntoIterator:
    type Item
    type Iter: needs Iterator[Item = Item]
    fn into_iter() owns this -> Iter

## `[STD-19]` (ODR-094) — a collection that can be built from values handed
## to it one at a time, which is what `collect` asks of its target. `Array`,
## `Set`, `Map` (from key-value pairs) and `String` (from characters or from
## strings) are; so is any type of a program's that implements it.
pub interface FromIterator[T]:
    [I: needs Iterator[Item = T]]
    static fn from_iter(owns it: I) -> Self

## `[STD-19]` — the adapters `Iterator`'s methods above make. Each holds the
## iterator it wraps and gives what `next` says, as that method describes.
[I]
pub struct Take:
    inner: I
    left: int

extend[I: needs Iterator] Take[I] implements Iterator:
    type Item = I.Item
    type Count = I.Count
    type Position = I.Position

    fn next() -> Option[I.Item] from inner:
        if this.left <= 0:
            return None
        this.left -= 1
        return this.inner.next()

    fn skip_front(n: I.Count) -> I.Count:
        wanted = n
        if n.exceeds(this.left):
            wanted = I.Count.of(max(this.left, 0))
        missed = this.inner.skip_front(wanted)
        gone = wanted.less(missed)
        this.left -= gone.to_int()
        return n.less(gone)

[I]
pub struct Skip:
    inner: I
    left: int

extend[I: needs Iterator] Skip[I] implements Iterator:
    type Item = I.Item
    type Count = I.Count
    type Position = I.Position

    fn next() -> Option[I.Item] from inner:
        if this.left > 0:
            first = I.Count.of(this.left)
            this.left = 0
            if this.inner.skip_front(first) > I.Count.of(0):
                return None
        return this.inner.next()

    fn skip_front(n: I.Count) -> I.Count:
        if this.left > 0:
            first = I.Count.of(this.left)
            this.left = 0
            if this.inner.skip_front(first) > I.Count.of(0):
                return n
        return this.inner.skip_front(n)

[I]
pub struct StepBy:
    inner: I
    ## The items left out between two given: the step less one.
    gap: int
    started: bool

extend[I: needs Iterator] StepBy[I] implements Iterator:
    type Item = I.Item
    type Count = I.Count
    type Position = I.Position

    fn next() -> Option[I.Item] from inner:
        if this.started:
            left = this.gap
            while left > 0:
                left -= 1
                if this.inner.next().is_none():
                    return None
        this.started = true
        return this.inner.next()

## Item `k` is numbered `start + k`, in `P`, which holds every number (`[STD-19]`): the
## iterator's `Position` (`int` for stored items, wider for a range of 64- or 128-bit numbers),
## or, at a call, the smallest type that holds the numbers the compiler knows it gives (the
## known-numbers rule), stepping up from `int` where they could pass its top. `number` is the
## next item's, and `past` says the last one given was numbered at `P`'s top, so only an
## iterator that gives more items than its `Count` holds can number past it.
[I, P]
pub struct Enumerate:
    inner: I
    number: P
    past: bool

extend[I: needs Iterator, P: needs ItemCount] Enumerate[I, P] implements Iterator:
    type Item = (P, I.Item)
    type Count = I.Count
    type Position = P

    fn next() -> Option[(P, I.Item)] from inner:
        match this.inner.next():
            Some(x):
                at = this.number
                if this.past:
                    at = at.plus(P.of(1))
                elif at.at_top():
                    this.past = true
                else:
                    this.number = at.plus(P.of(1))
                return Some((at, x))
            None:
                return None

[I, J]
pub struct Zip:
    a: I
    b: J

extend[I: needs Iterator, J: needs Iterator] Zip[I, J] implements Iterator:
    type Item = (I.Item, J.Item)
    type Count = I.Count
    type Position = I.Position

    fn next() -> Option[(I.Item, J.Item)]:
        match this.a.next():
            Some(x):
                match this.b.next():
                    Some(y):
                        return Some((x, y))
                    None:
                        return None
            None:
                return None

    fn skip_front(n: I.Count) -> I.Count:
        mine = this.a.skip_front(n).to_wide()
        wide = n.to_wide()
        theirs = J.Count.at_most(wide)
        missed = this.b.skip_front(theirs).to_wide() + (wide - theirs.to_wide())
        if missed > mine:
            return I.Count.of_wide(missed)
        return I.Count.of_wide(mine)

## `other` runs only once the first has run out (`done`), and the first is
## not asked again after it said `None`.
[I, J]
pub struct Chain:
    first: I
    other: J
    done: bool

extend[I: needs Iterator, J: needs Iterator[Item = I.Item]] Chain[I, J] implements Iterator:
    type Item = I.Item
    ## G8-4 — the two sides' items together, stepping up by need: `int` and `int` stay `int`
    ## (stored items and small ranges cannot come near its top); otherwise one level above the
    ## wider side's count, `u64` -> `u128` -> `u256`, so every count of the two has its type.
    type Count = CountSum[I.Count, J.Count]
    ## And the numbers `enumerate` gives it, which hold both sides'.
    type Position = PositionJoin[I.Position, J.Position]

    fn next() -> Option[I.Item]:
        if not this.done:
            match this.first.next():
                Some(x):
                    return Some(x)
                None:
                    this.done = true
        return this.other.next()

    fn skip_front(n: CountSum[I.Count, J.Count]) -> CountSum[I.Count, J.Count]:
        left = n.to_wide()
        if not this.done:
            mine = I.Count.at_most(left)
            missed = this.first.skip_front(mine).to_wide()
            left = missed + (left - mine.to_wide())
            if left == 0:
                return n.less(n)
            this.done = true
        theirs = J.Count.at_most(left)
        left = this.other.skip_front(theirs).to_wide() + (left - theirs.to_wide())
        return left.as_count[CountSum[I.Count, J.Count]]()

## `[STD-19]` — `it.copied()` and `it.cloned()` over an iterator of
## references `ref T`: the values they reach, copied or cloned, so the items
## are `T`s a program may keep (`xs.iter().copied().to_array()`). The checker
## makes these for an iterator whose items are references (the methods are
## not written here because `Item` being `ref T` is not a bound it can state).
[I, T]
pub struct Copied:
    inner: I

extend[T: needs Copy, I: needs Iterator[Item = ref T]] Copied[I, T] implements Iterator:
    type Item = T
    type Count = I.Count
    type Position = I.Position

    fn next() -> Option[T] from inner:
        match this.inner.next():
            Some(r):
                return Some(r)
            None:
                return None

    fn skip_front(n: I.Count) -> I.Count:
        return this.inner.skip_front(n)

[I, T]
pub struct Cloned:
    inner: I

extend[T: needs Clone, I: needs Iterator[Item = ref T]] Cloned[I, T] implements Iterator:
    type Item = T
    type Count = I.Count
    type Position = I.Position

    fn next() -> Option[T] from inner:
        match this.inner.next():
            Some(r):
                return Some(r.clone())
            None:
                return None

## `[STD-19]` — the adapters that hold a callable: `map`, `filter`,
## `filter_map`, `take_while`, `skip_while` and `inspect`. Each keeps the
## callable it was given by value, as `[CLO-14]`'s bound makes it, so a
## lambda that borrows keeps its borrows, and the adapter is a view while it
## does (`[TYP-15]`).
[I, R, F]
pub struct Map:
    inner: I
    f: F

extend[I: needs Iterator, R, F: needs fn(I.Item) -> R] Map[I, R, F] implements Iterator:
    type Item = R
    type Count = I.Count
    type Position = I.Position

    fn next() -> Option[R]:
        match this.inner.next():
            Some(x):
                return Some(this.f(x))
            None:
                return None

[I, F]
pub struct Filter:
    inner: I
    pred: F

extend[I: needs Iterator, F: needs fn(I.Item) -> bool] Filter[I, F] implements Iterator:
    type Item = I.Item
    type Count = I.Count
    type Position = I.Position

    fn next() -> Option[I.Item] from inner:
        while true:
            match this.inner.next():
                Some(x):
                    if this.pred(x):
                        return Some(x)
                None:
                    return None
        return None

[I, R, F]
pub struct FilterMap:
    inner: I
    f: F

extend[I: needs Iterator, R, F: needs fn(I.Item) -> Option[R]] FilterMap[I, R, F] implements Iterator:
    type Item = R
    type Count = I.Count
    type Position = I.Position

    fn next() -> Option[R]:
        while true:
            match this.inner.next():
                Some(x):
                    match this.f(x):
                        Some(y):
                            return Some(y)
                        None:
                            pass
                None:
                    return None
        return None

[I, F]
pub struct TakeWhile:
    inner: I
    pred: F
    done: bool

extend[I: needs Iterator, F: needs fn(I.Item) -> bool] TakeWhile[I, F] implements Iterator:
    type Item = I.Item
    type Count = I.Count
    type Position = I.Position

    fn next() -> Option[I.Item] from inner:
        if this.done:
            return None
        match this.inner.next():
            Some(x):
                if this.pred(x):
                    return Some(x)
                this.done = true
                return None
            None:
                return None

[I, F]
pub struct SkipWhile:
    inner: I
    pred: F
    started: bool

extend[I: needs Iterator, F: needs fn(I.Item) -> bool] SkipWhile[I, F] implements Iterator:
    type Item = I.Item
    type Count = I.Count
    type Position = I.Position

    fn next() -> Option[I.Item] from inner:
        if this.started:
            return this.inner.next()
        while true:
            match this.inner.next():
                Some(x):
                    if not this.pred(x):
                        this.started = true
                        return Some(x)
                None:
                    return None
        return None

[I, F]
pub struct Inspect:
    inner: I
    f: F

extend[I: needs Iterator, F: needs fn(I.Item)] Inspect[I, F] implements Iterator:
    type Item = I.Item
    type Count = I.Count
    type Position = I.Position

    fn next() -> Option[I.Item] from inner:
        match this.inner.next():
            Some(x):
                this.f(x)
                return Some(x)
            None:
                return None

## `[STD-19]` — `flat_map(f)`: the items of what `f` makes of each item.
## `inner` walks the current one; when it runs out, the next item is made into
## the next one, and when the items run out, so does this.
[I, U: needs IntoIterator, F]
pub struct FlatMap:
    outer: I
    f: F
    inner: Option[U.Iter]

extend[I: needs Iterator, U: needs IntoIterator, F: needs fn(I.Item) -> U] FlatMap[I, U, F] implements Iterator:
    type Item = U.Item
    ## G8-4 — every inner iterator's items together, stepping up by need: the level that holds the
    ## outer count times the inner one (an `int` by an `int` stays an `int`; `u256` the limit).
    type Count = CountProduct[I.Count, U.Iter.Count]
    ## And the numbers `enumerate` gives them.
    type Position = PositionOf[CountProduct[I.Count, U.Iter.Count]]

    fn next() -> Option[U.Item]:
        while true:
            match this.inner.as_mut():
                Some(it):
                    match it.next():
                        Some(x):
                            return Some(x)
                        None:
                            pass
                None:
                    pass
            match this.outer.next():
                Some(x):
                    this.inner = Some((this.f)(x).into_iter())
                None:
                    return None
        return None

## `[GRM-38]` — a generator expression's later `for` clause, `(e for x in xs for y in f(x))`: for
## each item `x` of `outer`, the pairs of `x` with each item of the iterator `f` makes of it. The
## iterator is held in place, so nothing is allocated, and what it iterates it borrows.
[I, X, U, F]
pub struct Nested:
    outer: I
    f: F
    item: Option[X]
    inner: Option[U]

extend[X, I: needs Iterator[Item = X], U: needs Iterator, F: needs fn(X) -> U] Nested[I, X, U, F] implements Iterator:
    type Item = (X, U.Item)
    ## G8-4 — every inner iterator's items together, as `flat_map`'s.
    type Count = CountProduct[I.Count, U.Count]
    ## And the numbers `enumerate` gives them.
    type Position = PositionOf[CountProduct[I.Count, U.Count]]

    fn next() -> Option[(X, U.Item)]:
        while true:
            match this.item:
                Some(x):
                    match this.inner.as_mut():
                        Some(it):
                            match it.next():
                                Some(y):
                                    return Some((x, y))
                                None:
                                    pass
                        None:
                            pass
                None:
                    pass
            match this.outer.next():
                Some(x):
                    this.item = Some(x)
                    this.inner = Some(this.f(x))
                None:
                    return None
        return None

## `[STD-19]` — `flatten()`: the items of each item, as `flat_map` gives them
## with nothing made of the items first. `U` is the items' type, named so its
## bound can be stated.
[I, U: needs IntoIterator]
pub struct Flatten:
    outer: I
    inner: Option[U.Iter]

extend[U: needs IntoIterator, I: needs Iterator[Item = U]] Flatten[I, U] implements Iterator:
    type Item = U.Item
    ## G8-4 — every inner iterator's items together, stepping up by need: the level that holds the
    ## outer count times the inner one (an `int` by an `int` stays an `int`; `u256` the limit).
    type Count = CountProduct[I.Count, U.Iter.Count]
    ## And the numbers `enumerate` gives them.
    type Position = PositionOf[CountProduct[I.Count, U.Iter.Count]]

    fn next() -> Option[U.Item]:
        while true:
            match this.inner.as_mut():
                Some(it):
                    match it.next():
                        Some(x):
                            return Some(x)
                        None:
                            pass
                None:
                    pass
            match this.outer.next():
                Some(x):
                    this.inner = Some(x.into_iter())
                None:
                    return None
        return None

## `[STD-19]` (ODR-094) — `peekable()`: the items of the iterator it wraps,
## with the next one taken early by `peek` and kept (`looked`) until `next`
## gives it.
[I: needs Iterator]
pub struct Peekable:
    inner: I
    peeked: Option[I.Item]
    looked: bool

extend[I: needs Iterator] Peekable[I] implements Iterator:
    type Item = I.Item
    type Count = I.Count
    type Position = I.Position

    fn next() -> Option[I.Item]:
        if this.looked:
            this.looked = false
            return this.peeked.take()
        return this.inner.next()

extend[I: needs Iterator] Peekable[I]:
    ## The next item, left where it is: the following `next` gives it.
    pub fn peek() -> Option[ref I.Item]:
        if not this.looked:
            this.peeked = this.inner.next()
            this.looked = true
        return this.peeked.as_ref()

    ## The next item, left where it is, to change in place.
    pub fn peek_mut() -> Option[ref mut I.Item]:
        if not this.looked:
            this.peeked = this.inner.next()
            this.looked = true
        return this.peeked.as_mut()

    ## The next item when `pred` holds for it; otherwise it stays.
    [F: needs fn(ref I.Item) -> bool]
    pub fn next_if(owns pred: F) -> Option[I.Item]:
        take = false
        match this.peek():
            Some(x):
                take = pred(x)
            None:
                pass
        if take:
            return this.next()
        return None

extend[T: needs Eq, I: needs Iterator[Item = T]] Peekable[I]:
    ## The next item when it equals `expected`; otherwise it stays.
    pub fn next_if_eq(expected: T) -> Option[T]:
        take = false
        match this.peek():
            Some(x):
                take = x == expected
            None:
                pass
        if take:
            return this.next()
        return None

## `[STD-19]` (ODR-091) — an iterator run backwards: its `next` is the one it
## wraps's `next_back`, and the other way round.
[I]
pub struct Rev:
    inner: I

extend[I: needs DoubleEndedIterator] Rev[I] implements Iterator:
    type Item = I.Item
    type Count = I.Count
    type Position = I.Position

    fn next() -> Option[I.Item] from inner:
        return this.inner.next_back()

    fn skip_front(n: I.Count) -> I.Count:
        return this.inner.skip_back(n)

extend[I: needs DoubleEndedIterator] Rev[I] implements DoubleEndedIterator:
    fn next_back() -> Option[I.Item] from inner:
        return this.inner.next()

    fn skip_back(n: I.Count) -> I.Count:
        return this.inner.skip_front(n)

extend[I: needs (DoubleEndedIterator, ExactSizeIterator)] Rev[I] implements ExactSizeIterator:
    fn len() -> I.Count:
        return this.inner.len()

## ODR-091 — which adapters run backwards, and know their length. One whose
## last item depends on how many there are (`take`, `skip`, `step_by`,
## `enumerate`, `zip`) runs backwards when the iterator below it knows its
## length; `copied`, `cloned` and `chain` run backwards when theirs do.
extend[I: needs ExactSizeIterator] Take[I] implements ExactSizeIterator:
    fn len() -> I.Count:
        left = this.inner.len()
        if left.exceeds(this.left):
            return I.Count.of(max(this.left, 0))
        return left

## G8-4 — the items after the first `left` are left out from the back at once (`nth_back`), so a
## range too long for an `int` to count gives its first items backwards too.
extend[I: needs (DoubleEndedIterator, ExactSizeIterator)] Take[I] implements DoubleEndedIterator:
    fn next_back() -> Option[I.Item] from inner:
        if this.left <= 0:
            return None
        left = this.inner.len()
        keep = this.left
        this.left -= 1
        if left.exceeds(keep):
            return this.inner.nth_back(left.less(I.Count.of(keep)))
        return this.inner.next_back()

    fn skip_back(n: I.Count) -> I.Count:
        if this.left <= 0:
            return n
        wanted = n
        if n.exceeds(this.left):
            wanted = I.Count.of(this.left)
        held = this.inner.len()
        if held.exceeds(this.left):
            this.inner.skip_back(held.less(I.Count.of(this.left)))
        missed = this.inner.skip_back(wanted)
        gone = wanted.less(missed)
        this.left -= gone.to_int()
        return n.less(gone)

extend[I: needs ExactSizeIterator] Skip[I] implements ExactSizeIterator:
    fn len() -> I.Count:
        left = this.inner.len()
        if left.exceeds(this.left):
            return left.less(I.Count.of(this.left))
        return I.Count.of(0)

extend[I: needs (DoubleEndedIterator, ExactSizeIterator)] Skip[I] implements DoubleEndedIterator:
    fn next_back() -> Option[I.Item] from inner:
        if not this.inner.len().exceeds(this.left):
            return None
        return this.inner.next_back()

    fn skip_back(n: I.Count) -> I.Count:
        held = this.inner.len()
        if not held.exceeds(this.left):
            return n
        room = held.less(I.Count.of(this.left))
        wanted = n
        if n > room:
            wanted = room
        missed = this.inner.skip_back(wanted)
        return n.less(wanted.less(missed))

## The items a `step_by(k)` gives are at `0, k, 2k, …` of those left before
## the first, then at `k - 1, 2k - 1, …` after it (`gap` is `k - 1`).
extend[I: needs ExactSizeIterator] StepBy[I] implements ExactSizeIterator:
    fn len() -> I.Count:
        left = this.inner.len()
        step = this.gap + 1
        if this.started:
            return left.div_by(step)
        if not left.exceeds(0):
            return I.Count.of(0)
        one = I.Count.of(1)
        return one.plus(left.less(one).div_by(step))

extend[I: needs (DoubleEndedIterator, ExactSizeIterator)] StepBy[I] implements DoubleEndedIterator:
    fn next_back() -> Option[I.Item] from inner:
        left = this.inner.len()
        step = this.gap + 1
        extra = 0
        if this.started:
            if not left.exceeds(step - 1):
                return None
            extra = left.rem_by(step)
        else:
            if not left.exceeds(0):
                return None
            extra = left.less(I.Count.of(1)).rem_by(step)
        if extra > 0:
            return this.inner.nth_back(I.Count.of(extra))
        return this.inner.next_back()

extend[I: needs ExactSizeIterator, P: needs ItemCount] Enumerate[I, P] implements ExactSizeIterator:
    fn len() -> I.Count:
        return this.inner.len()

## The last item left is numbered the next one's number plus the items
## between.
extend[I: needs (DoubleEndedIterator, ExactSizeIterator), P: needs ItemCount] Enumerate[I, P] implements DoubleEndedIterator:
    fn next_back() -> Option[(P, I.Item)] from inner:
        left = this.inner.len()
        match this.inner.next_back():
            Some(x):
                at = this.number.plus(P.of_wide(left.to_wide() - 1))
                if this.past:
                    at = at.plus(P.of(1))
                return Some((at, x))
            None:
                return None

    ## The items left out need no numbers: `P` holds every number they could have had
    ## (`[STD-19]`).
    fn skip_back(n: I.Count) -> I.Count:
        return this.inner.skip_back(n)

extend[I: needs ExactSizeIterator, J: needs ExactSizeIterator] Zip[I, J] implements ExactSizeIterator:
    fn len() -> I.Count:
        mine = this.a.len()
        theirs = this.b.len().to_wide()
        if theirs < mine.to_wide():
            return I.Count.of_wide(theirs)
        return mine

## G8-4 — the longer side's extra items left out at once, so both have as many left.
extend[I: needs (DoubleEndedIterator, ExactSizeIterator), J: needs (DoubleEndedIterator, ExactSizeIterator)] Zip[I, J]:
    fn even_up():
        mine = this.a.len().to_wide()
        theirs = this.b.len().to_wide()
        if mine > theirs:
            this.a.skip_back(I.Count.of_wide(mine - theirs))
        if theirs > mine:
            this.b.skip_back(J.Count.of_wide(theirs - mine))

## The longer is cut to the shorter's length from the back first, so the
## pairs are the ones `next` would make.
extend[I: needs (DoubleEndedIterator, ExactSizeIterator), J: needs (DoubleEndedIterator, ExactSizeIterator)] Zip[I, J] implements DoubleEndedIterator:
    fn next_back() -> Option[(I.Item, J.Item)]:
        this.even_up()
        match this.a.next_back():
            Some(x):
                match this.b.next_back():
                    Some(y):
                        return Some((x, y))
                    None:
                        return None
            None:
                return None

    fn skip_back(n: I.Count) -> I.Count:
        this.even_up()
        missed = this.a.skip_back(n)
        this.b.skip_back(J.Count.at_most(n.to_wide()))
        return missed

extend[I: needs ExactSizeIterator, J: needs ExactSizeIterator[Item = I.Item]] Chain[I, J] implements ExactSizeIterator:
    fn len() -> CountSum[I.Count, J.Count]:
        rest = this.other.len().as_count[CountSum[I.Count, J.Count]]()
        if this.done:
            return rest
        return this.first.len().as_count[CountSum[I.Count, J.Count]]().plus(rest)

## Backwards the second runs first; the first is not asked once it said
## `None` from the front.
extend[I: needs DoubleEndedIterator, J: needs DoubleEndedIterator[Item = I.Item]] Chain[I, J] implements DoubleEndedIterator:
    fn next_back() -> Option[I.Item]:
        match this.other.next_back():
            Some(x):
                return Some(x)
            None:
                if this.done:
                    return None
                return this.first.next_back()

    fn skip_back(n: CountSum[I.Count, J.Count]) -> CountSum[I.Count, J.Count]:
        left = n.to_wide()
        theirs = J.Count.at_most(left)
        left = this.other.skip_back(theirs).to_wide() + (left - theirs.to_wide())
        if left > 0 and not this.done:
            mine = I.Count.at_most(left)
            left = this.first.skip_back(mine).to_wide() + (left - mine.to_wide())
        return left.as_count[CountSum[I.Count, J.Count]]()

extend[T: needs Copy, I: needs ExactSizeIterator[Item = ref T]] Copied[I, T] implements ExactSizeIterator:
    fn len() -> I.Count:
        return this.inner.len()

extend[T: needs Copy, I: needs DoubleEndedIterator[Item = ref T]] Copied[I, T] implements DoubleEndedIterator:
    fn next_back() -> Option[T] from inner:
        match this.inner.next_back():
            Some(r):
                return Some(r)
            None:
                return None

    fn skip_back(n: I.Count) -> I.Count:
        return this.inner.skip_back(n)

extend[T: needs Clone, I: needs ExactSizeIterator[Item = ref T]] Cloned[I, T] implements ExactSizeIterator:
    fn len() -> I.Count:
        return this.inner.len()

extend[T: needs Clone, I: needs DoubleEndedIterator[Item = ref T]] Cloned[I, T] implements DoubleEndedIterator:
    fn next_back() -> Option[T] from inner:
        match this.inner.next_back():
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
[T]
pub struct Range:
    pub start: T
    pub end: T

@derive(Copy)
[T]
pub struct RangeInclusive:
    pub start: T
    pub end: T

@derive(Copy)
[T]
pub struct RangeFrom:
    pub start: T

@derive(Copy)
[T]
pub struct RangeTo:
    pub end: T

## `[STD-19]`, `[CTL-3b]` — a range of integers is `Iterable`, as Python's
## `range` is: `r.iter()` counts through a copy of its bounds, so the range is
## left as it was, and `(a..b).skip(1)` is `(a..b).iter().skip(1)`. A `for`
## over a chain of adapters on one is a counted loop.
extend[T: needs Integer] Range[T]:
    pub fn iter() -> RangeIter[T]:
        return RangeIter(this.start, this.end)

    ## `[STD-26]` — Python's stepped range, which `range(start, stop, step)` is as a value: from
    ## `start` by `step` up to, not including, `end`; a negative `step` counts down. A zero step
    ## is the caller's bug.
    pub fn stepped(step: T) -> StepRange[T]:
        zero = T.default()
        if step == zero:
            panic("range() step must not be zero")
        if step > zero:
            return StepRange(this.start, this.end, step, zero.distance_to(step))
        return StepRange(this.start, this.end, step, step.distance_to(zero))

extend[T: needs Integer] RangeInclusive[T]:
    pub fn iter() -> RangeInclusiveIter[T]:
        return RangeInclusiveIter(this.start, this.end, this.end < this.start)

extend[T: needs Integer] RangeFrom[T]:
    pub fn iter() -> RangeFromIter[T]:
        return RangeFromIter(this.start, false)

## `[STD-26]` — the iterator `Range.stepped` gives: the next value `at`, the `stop` it never
## reaches, and the `step` as written. Each step is measured against what is left before it is
## taken, so it never steps past the type's ends; once done, `at` is `stop`. A `for` loop over one
## is a counted loop from `at` (`[CTL-3b]`).
[T: needs Integer]
pub struct StepRange:
    pub at: T
    pub stop: T
    pub step: T
    stride: T.SpanCount

extend[T: needs Integer] StepRange[T] implements Iterator:
    type Item = T
    type Count = T.SpanCount
    type Position = T.Position

    fn next() -> Option[T]:
        here = this.at
        if this.step > T.default():
            if not (here < this.stop):
                return None
            if here.distance_to(this.stop) > this.stride:
                this.at = here.forward_by(this.stride)
            else:
                this.at = this.stop
        else:
            if not (this.stop < here):
                return None
            if this.stop.distance_to(here) > this.stride:
                this.at = here.back_by(this.stride)
            else:
                this.at = this.stop
        return Some(here)

## `[CTL-1]` (D-444) — a range is iterable, and consumed as a value its
## items are its `iter()`'s: `flat_map(fn(x: int) => 0..x)`.
extend[T: needs Integer] Range[T] implements IntoIterator:
    type Item = T
    type Iter = RangeIter[T]

    fn into_iter() owns this -> RangeIter[T]:
        return this.iter()

extend[T: needs Integer] RangeInclusive[T] implements IntoIterator:
    type Item = T
    type Iter = RangeInclusiveIter[T]

    fn into_iter() owns this -> RangeInclusiveIter[T]:
        return this.iter()

extend[T: needs Integer] RangeFrom[T] implements IntoIterator:
    type Item = T
    type Iter = RangeFromIter[T]

    fn into_iter() owns this -> RangeFromIter[T]:
        return this.iter()

## The values from `at` up to, not including, `end`.
[T]
pub struct RangeIter:
    at: T
    end: T

extend[T: needs Integer] RangeIter[T] implements Iterator:
    type Item = T
    type Count = T.SpanCount
    type Position = T.Position

    fn next() -> Option[T]:
        if this.at < this.end:
            here = this.at
            this.at = here.successor()
            return Some(here)
        return None

    fn skip_front(n: T.SpanCount) -> T.SpanCount:
        if this.at < this.end:
            gap = this.at.distance_to(this.end)
            if gap > n:
                this.at = this.at.forward_by(n)
                return T.SpanCount.of(0)
            this.at = this.end
            return n.less(gap)
        return n

## ODR-091 — backwards from `end`, which moves down to meet `at`; G8-4 — `nth_back` moves it
## past `n` values at once.
extend[T: needs Integer] RangeIter[T] implements DoubleEndedIterator:
    fn next_back() -> Option[T]:
        if this.at < this.end:
            this.end = this.end.predecessor()
            return Some(this.end)
        return None

    fn skip_back(n: T.SpanCount) -> T.SpanCount:
        if this.at < this.end:
            gap = this.at.distance_to(this.end)
            if gap > n:
                this.end = this.end.back_by(n)
                return T.SpanCount.of(0)
            this.end = this.at
            return n.less(gap)
        return n

extend[T: needs Integer] RangeIter[T] implements ExactSizeIterator:
    fn len() -> T.SpanCount:
        if this.at < this.end:
            return this.at.distance_to(this.end)
        return T.SpanCount.of(0)

## The values from `at` up to and including `end`; `done` once `end` is
## given, so a range ending at the type's top never steps past it.
[T]
pub struct RangeInclusiveIter:
    at: T
    end: T
    done: bool

extend[T: needs Integer] RangeInclusiveIter[T] implements Iterator:
    type Item = T
    type Count = T.FullCount
    type Position = T.Position

    fn next() -> Option[T]:
        if this.done:
            return None
        here = this.at
        if here < this.end:
            this.at = here.successor()
        else:
            this.done = true
        return Some(here)

    fn skip_front(n: T.FullCount) -> T.FullCount:
        if this.done:
            return n
        count = T.widen_count(this.at.distance_to(this.end)).plus(T.FullCount.of(1))
        if count > n:
            this.at = this.at.forward_by(T.narrow_count(n))
            return T.FullCount.of(0)
        this.done = true
        return n.less(count)

## ODR-091 — backwards from `end`; `done` once the two meet, so a range
## starting at the type's bottom never steps below it.
extend[T: needs Integer] RangeInclusiveIter[T] implements DoubleEndedIterator:
    fn next_back() -> Option[T]:
        if this.done:
            return None
        here = this.end
        if this.at < here:
            this.end = here.predecessor()
        else:
            this.done = true
        return Some(here)

    ## G8-4 — `end` moved past `n` values at once; an `n` below the count leaves `at` at least.
    fn skip_back(n: T.FullCount) -> T.FullCount:
        if this.done:
            return n
        count = T.widen_count(this.at.distance_to(this.end)).plus(T.FullCount.of(1))
        if count > n:
            this.end = this.end.back_by(T.narrow_count(n))
            return T.FullCount.of(0)
        this.done = true
        return n.less(count)

extend[T: needs Integer] RangeInclusiveIter[T] implements ExactSizeIterator:
    fn len() -> T.FullCount:
        if this.done:
            return T.FullCount.of(0)
        return T.widen_count(this.at.distance_to(this.end)).plus(T.FullCount.of(1))

## The values from `at` on, the type's top included; the value after the top is an overflow
## (`[TYP-8]`), which panics when it is asked for, as in a `for` over `a..` (`[CTL-3]`, D-526).
## `given` says `at` has been given.
[T]
pub struct RangeFromIter:
    at: T
    given: bool

extend[T: needs Integer] RangeFromIter[T] implements Iterator:
    type Item = T
    ## G8-4 — it gives the values up to the type's top at most.
    type Count = T.FullCount
    type Position = T.Position

    fn next() -> Option[T]:
        if this.given:
            this.at = this.at.successor()
        this.given = true
        return Some(this.at)

    ## G8-4 — the values left out at once; leaving out the value after the top panics, as asking
    ## for it would.
    fn skip_front(n: T.FullCount) -> T.FullCount:
        if not n.exceeds(0):
            return n
        if this.given:
            this.at = this.at.successor()
        this.given = true
        left = this.at.count_to_top()
        if n > left:
            this.at = this.at.forward_by(T.narrow_count(left.less(T.FullCount.of(1))))
            this.at = this.at.successor()
        this.at = this.at.forward_by(T.narrow_count(n.less(T.FullCount.of(1))))
        return T.FullCount.of(0)

## Part IV §8 also declares `Hash`, which `std.collections` holds with the
## hashers.

## `[ERR-4]` (ODR-025) — the `Option` and `Result` methods that take a
## function. `x.map(f)` is a call of `option_map(x, f)`: the compiler routes the
## method here, so each is an ordinary generic function whose callback is
## checked, inferred and borrow-checked like any other. Each consumes the
## receiver and calls `f` at most once, before it returns, so `f` is a `once fn`: a closure may give
## away what it captured. A payload passed to `f` is moved in (`owned`), except `filter`'s, which it
## must give back. A named callable that may be called again is lent, the caller keeping it
## (`[STD-19]`, ODR-240); a lambda written at the call, or one that gives a capture away, is given.

[T, U, Call: needs once fn(owns T) -> U]
fn option_map(owns o: Option[T], owns f: Call) -> Option[U]:
    match owns o:
        Some(v):
            return Some(f(v))
        None:
            return None

[T, U, Call: needs once fn(owns T) -> Option[U]]
fn option_and_then(owns o: Option[T], owns f: Call) -> Option[U]:
    match owns o:
        Some(v):
            return f(v)
        None:
            return None

[T, Call: needs once fn(T) -> bool]
fn option_filter(owns o: Option[T], owns f: Call) -> Option[T]:
    match owns o:
        Some(v):
            if f(v):
                return Some(v)
            return None
        None:
            return None

[T, Call: needs once fn() -> Option[T]]
fn option_or_else(owns o: Option[T], owns f: Call) -> Option[T]:
    match owns o:
        Some(v):
            return Some(v)
        None:
            return f()

[T, Call: needs once fn() -> T]
fn option_unwrap_or_else(owns o: Option[T], owns f: Call) -> T:
    match owns o:
        Some(v):
            return v
        None:
            return f()

[T, E, Call: needs once fn() -> E]
fn option_ok_or_else(owns o: Option[T], owns f: Call) -> Result[T, E]:
    match owns o:
        Some(v):
            return Ok(v)
        None:
            return Err(f())

[T, E, U, Call: needs once fn(owns T) -> U]
fn result_map(owns r: Result[T, E], owns f: Call) -> Result[U, E]:
    match owns r:
        Ok(v):
            return Ok(f(v))
        Err(e):
            return Err(e)

[T, E, F, Call: needs once fn(owns E) -> F]
fn result_map_err(owns r: Result[T, E], owns f: Call) -> Result[T, F]:
    match owns r:
        Ok(v):
            return Ok(v)
        Err(e):
            return Err(f(e))

[T, E, U, Call: needs once fn(owns T) -> Result[U, E]]
fn result_and_then(owns r: Result[T, E], owns f: Call) -> Result[U, E]:
    match owns r:
        Ok(v):
            return f(v)
        Err(e):
            return Err(e)

[T, E, F, Call: needs once fn(owns E) -> Result[T, F]]
fn result_or_else(owns r: Result[T, E], owns f: Call) -> Result[T, F]:
    match owns r:
        Ok(v):
            return Ok(v)
        Err(e):
            return f(e)

[T, E, Call: needs once fn(owns E) -> T]
fn result_unwrap_or_else(owns r: Result[T, E], owns f: Call) -> T:
    match owns r:
        Ok(v):
            return v
        Err(e):
            return f(e)

## IX.1 (ODR-241) — the compiler's: what the table of a `dyn I` whose interface requires `Eq` calls
## to compare two values of the type it was made for, after it has found both are of that type:
## `a == b` as the type writes it.
[T: needs Eq]
fn dyn_eq(a: T, b: T) -> bool:
    return a == b

## `[TYP-36]` — the table's `Default` column: zero, `0.0`, `false`, `'\0'`,
## empty text, an empty `Array` and `None`.

extend i8 implements Default:
    static fn default() -> i8:
        return 0

extend i16 implements Default:
    static fn default() -> i16:
        return 0

extend i32 implements Default:
    static fn default() -> i32:
        return 0

extend i64 implements Default:
    static fn default() -> i64:
        return 0

extend isize implements Default:
    static fn default() -> isize:
        return 0

extend u8 implements Default:
    static fn default() -> u8:
        return 0

extend u16 implements Default:
    static fn default() -> u16:
        return 0

extend u32 implements Default:
    static fn default() -> u32:
        return 0

extend u64 implements Default:
    static fn default() -> u64:
        return 0

extend usize implements Default:
    static fn default() -> usize:
        return 0

extend i128 implements Default:
    static fn default() -> i128:
        return 0

extend u128 implements Default:
    static fn default() -> u128:
        return 0

extend f16 implements Default:
    static fn default() -> f16:
        return 0.0

extend f32 implements Default:
    static fn default() -> f32:
        return 0.0

extend f64 implements Default:
    static fn default() -> f64:
        return 0.0

extend bool implements Default:
    static fn default() -> bool:
        return false

extend char implements Default:
    static fn default() -> char:
        return '\0'

extend StrView implements Default:
    static fn default() -> StrView:
        return ""

## The characters, in order (ODR-094).
extend String implements FromIterator[char]:
    [I: needs Iterator[Item = char]]
    static fn from_iter(owns it: I) -> String:
        out = String()
        for c in it:
            out.push(c)
        return out

## The strings, one after another (ODR-094).
extend String implements FromIterator[String]:
    [I: needs Iterator[Item = String]]
    static fn from_iter(owns it: I) -> String:
        out = String()
        for s in it:
            out += s
        return out

extend String implements Default:
    ## It allocates the new text (`[STD-1]`).
    static fn default() -> String:
        return String.from("")

extend[T] Array[T] implements Default:
    static fn default() -> Array[T]:
        return []

extend[T] Option[T] implements Default:
    static fn default() -> Option[T]:
        return None

## `[ERR-4]` — `take` and `replace`, written in Ember with `mem.replace`.
extend[T] Option[T]:
    ## The payload, leaving `None`.
    pub fn take() -> Option[T]:
        return mem.replace(ref mut this, None)

    ## The payload, leaving `Some(value)`.
    pub fn replace(owns value: T) -> Option[T]:
        return mem.replace(ref mut this, Some(value))

## `[STD-15]` — the `Array` methods written in Ember (`[GRM-34]`). A method
## the compiler knows by a name comes first (`[TYP-24]`); these are found
## after it, and one a program never calls is not emitted (`[COST-1]`).
## `for x in owned xs:` (`[CTL-1]`): the elements in order, owned. The array
## is reversed once and popped, so each element is taken without a copy, and
## those left when a loop ends early are dropped with the iterator.
[T]
pub struct ArrayIntoIter:
    rest: Array[T]

    pub fn next() -> Option[T]:
        return this.rest.pop()

extend[T] ArrayIntoIter[T] implements Iterator:
    type Item = T

## The items in order (ODR-094).
extend[T] Array[T] implements FromIterator[T]:
    [I: needs Iterator[Item = T]]
    static fn from_iter(owns it: I) -> Array[T]:
        out: Array[T] = []
        for x in it:
            out.push(owns x)
        return out

extend[T] Array[T] implements IntoIterator:
    type Item = T
    type Iter = ArrayIntoIter[T]

    fn into_iter() owns this -> ArrayIntoIter[T]:
        rest = this
        rest.reverse()
        return ArrayIntoIter(rest)

extend[T] Array[T]:
    ## Keeps the elements `keep` is true of, in their order.
    [F: needs fn(T) -> bool]
    pub fn retain(owns keep: F):
        kept = 0
        for i in range(this.len()):
            if keep(this[i]):
                this.swap(kept, i)
                kept += 1
        this.truncate(kept)

    ## Sorts by `cmp`, which says how two elements order. Stable: elements
    ## `cmp` finds equal keep their order (ODR-031).
    [F: needs fn(T, T) -> Ordering]
    pub fn sort_by(owns cmp: F):
        order = stable_order(this, cmp)
        this.apply_order(ref mut order)

    ## Sorts by `key(x)`, calling `key` once for each element, as Python's
    ## `key=` does. Stable (ODR-031).
    [K: needs Ord, F: needs fn(T) -> K]
    pub fn sort_by_key(owns key: F):
        keys: Array[K] = []
        for i in range(this.len()):
            keys.push(owns key(this[i]))
        order = stable_order(keys, owns fn(a, b) => a.cmp(b))
        this.apply_order(ref mut order)

    ## `[STD-15]` (ODR-068) — mutable references to the elements at `i` and
    ## `j` at once, or `None` when either index is out of range or they are
    ## the same: the mutable view's `get_pair_mut`.
    pub fn get_pair_mut(i: int, j: int) -> Option[(ref mut T, ref mut T)]:
        return this.as_mut_span().get_pair_mut(i, j)

    ## Puts the element at `order[i]` at `i`, following each cycle of the
    ## permutation with `swap`: no element is copied or moved out, so any `T`
    ## sorts.
    fn apply_order(ref mut order: Array[int]):
        for start in range(this.len()):
            at = start
            while order[at] != start:
                next = order[at]
                this.swap(at, next)
                order[at] = at
                at = next
            order[at] = at

## `[SOA-3]` — an `SoA`'s `retain` and `sort_by_key`, which move whole elements (every column
## together) with `swap` and drop with `truncate`, the compiler's own `SoA` operations for std.
extend[T] SoA[T]:
    ## Keeps the elements `keep` is true of, in their order: one dropped where it stands, one kept
    ## moved down over the first empty slot, so each element is moved at most once.
    [F: needs fn(SoARef[T]) -> bool]
    pub fn retain(owns keep: F):
        kept = 0
        for i in range(this.len()):
            if keep(this[i]):
                if kept != i:
                    this.move_slot(i, kept)
                kept += 1
            else:
                this.drop_at(i)
        this.set_len(kept)

    ## Sorts the elements by `key` of each, calling it once per element. Stable: elements with
    ## equal keys keep their order.
    [K: needs Ord, F: needs fn(SoARef[T]) -> K]
    pub fn sort_by_key(owns key: F):
        keys: Array[K] = []
        for i in range(this.len()):
            keys.push(owns key(this[i]))
        order = stable_order(keys, owns fn(a, b) => a.cmp(b))
        for start in range(this.len()):
            at = start
            while order[at] != start:
                next = order[at]
                this.swap(at, next)
                order[at] = at
                at = next
            order[at] = at

extend[T] MutSpan[T]:
    ## `[SPN-5]` (ODR-068) — mutable references to the elements at `i` and
    ## `j` at once, or `None` when either index is out of range or they are
    ## the same. The references come from `split_at`'s two halves, so they
    ## are disjoint before either exists.
    pub fn get_pair_mut(i: int, j: int) -> Option[(ref mut T, ref mut T)]:
        n = this.len()
        if i < 0 or j < 0 or i >= n or j >= n or i == j:
            return None
        if i < j:
            (low, high) = this.split_at(j)
            return Some((ref mut low[i], ref mut high[0]))
        (low, high) = this.split_at(i)
        return Some((ref mut high[0], ref mut low[j]))

## `[STR-5]` — element-wise equality through `T: Eq`. The checker routes a
## comparison of sequences here when the element type holds a written `eq`.
extend[T: needs Eq] Span[T]:
    fn eq_elements(other: Span[T]) -> bool:
        if this.len() != other.len():
            return false
        for i in range(this.len()):
            if this[i] != other[i]:
                return false
        return true

## `[TYP-36]` — the lexicographic order through `T: Ord`: the first elements that differ decide,
## and a sequence that is the start of the other is the smaller. The checker routes `cmp`, `<`,
## `sort` and `sorted` of an `Array` or a `[T; N]` here.
extend[T: needs Ord] Span[T]:
    fn cmp_elements(other: Span[T]) -> Ordering:
        n = this.len()
        if other.len() < n:
            n = other.len()
        for i in range(n):
            order = this[i].cmp(other[i])
            if order != Ordering.Equal:
                return order
        return this.len().cmp(other.len())

extend[T: needs Eq] Array[T]:
    ## Drops each element equal to the one kept before it, so a sorted array
    ## keeps one of each value.
    pub fn dedup():
        if this.len() < 2:
            return
        kept = 1
        for i in range(1, this.len()):
            if this[i] != this[kept - 1]:
                this.swap(kept, i)
                kept += 1
        this.truncate(kept)

extend[T: needs Ord] Array[T]:
    ## In a sorted array: `Ok` of an index holding `x`, or `Err` of the index
    ## where `x` would go to keep the array sorted.
    pub fn binary_search(x: T) -> Result[int, int]:
        lo = 0
        hi = this.len()
        while lo < hi:
            mid = (lo + hi) // 2
            match this[mid].cmp(x):
                Ordering.Less: lo = mid + 1
                Ordering.Equal: return Ok(mid)
                Ordering.Greater: hi = mid
        return Err(lo)

    ## `sort` for an element type with an `Ord` of its own. The compiler's
    ## `sort` orders numbers and text itself and routes the rest here (D-256).
    fn sort_ord():
        order = stable_order(this, owns fn(a, b) => a.cmp(b))
        this.apply_order(ref mut order)

extend[T: needs (Ord, Clone)] Array[T]:
    ## `sorted` for what the compiler's own `sorted` does not take: elements
    ## with an `Ord` of their own, or that are not `Copy` (D-256).
    fn sorted_ord() -> Array[T]:
        out = this.clone()
        out.sort_ord()
        return out

    ## `[STD-26]` — `sorted(it, reverse=r)`: a copy of these elements in order, the largest first
    ## when `reverse`. Stable both ways: elements that compare equal keep their order (ODR-031).
    fn sorted_reverse_if(reverse: bool) -> Array[T]:
        out = this.clone()
        if reverse:
            order = stable_order(out, owns fn(a, b) => b.cmp(a))
            out.apply_order(ref mut order)
        else:
            out.sort_ord()
        return out

extend[T: needs Clone] Array[T]:
    ## `[STD-26]` — `sorted(it, key=f, reverse=r)`: a copy of these elements ordered by `key`, each
    ## element's key computed once, as Python's `key=` does, the largest first when `reverse`.
    ## Stable both ways: elements whose keys are equal keep their order (ODR-031).
    [K: needs Ord, F: needs fn(T) -> K]
    fn sorted_by_key_if(owns key: F, reverse: bool) -> Array[T]:
        keys: Array[K] = []
        for i in range(this.len()):
            keys.push(owns key(this[i]))
        out = this.clone()
        if reverse:
            order = stable_order(keys, owns fn(a, b) => b.cmp(a))
            out.apply_order(ref mut order)
        else:
            order = stable_order(keys, owns fn(a, b) => a.cmp(b))
            out.apply_order(ref mut order)
        return out

## `[STD-15]` — the order a stable sort puts `xs` in, as indices: a bottom-up
## merge that takes from the right run only when `cmp` finds its element
## less. Each pass writes every index once, whatever `cmp` answers, so an
## inconsistent order can only permute the elements, never lose or repeat one
## (`[HASH-3]`).
[T, F: needs fn(T, T) -> Ordering]
fn stable_order(xs: Array[T], owns cmp: F) -> Array[int]:
    n = xs.len()
    order: Array[int] = []
    merged: Array[int] = []
    for i in range(n):
        order.push(owns i)
        merged.push(owns i)
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

extend[T: needs Display] Array[T]:
    ## The elements' text with `sep` between each two, as Python's
    ## `sep.join(xs)`.
    pub fn join(sep: StrView) -> String:
        out = String.from("")
        for i in range(this.len()):
            if i > 0:
                out += sep
            out += f"{this[i]}"
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
interface Integer: needs (Hash, Default, Ord, Copy):
    ## `[STD-19]` (G8-4) — what a `..` range of these numbers counts its values in, the
    ## smallest type that holds every count it can have: `int` for numbers of 8, 16 or 32
    ## bits, `u64` for 64 bits, `u128` for 128 bits.
    type SpanCount: needs ItemCount
    ## And a `..=` range, which can hold one value more: `int`, `u128`, `u256`.
    type FullCount: needs ItemCount
    ## What `enumerate` numbers such a range with, from a start that may be negative: `int`,
    ## `i128`, `i256`.
    type Position: needs ItemCount

    ## The next integer; past the type's top it panics, as `+ 1` does.
    fn successor() -> Self

    ## The integer before; past the type's bottom it panics, as `- 1` does.
    fn predecessor() -> Self

    ## How many integers from this one up to, not including, `end`, which is
    ## not below it: a `..` range's length.
    fn distance_to(end: Self) -> SpanCount

    ## The integer `n` before this one, which the type holds.
    fn back_by(n: SpanCount) -> Self

    ## The integer `n` after this one, which the type holds.
    fn forward_by(n: SpanCount) -> Self

    ## How many integers from this one up to the type's top, both included: what `this..` holds
    ## (D-526).
    fn count_to_top() -> FullCount

    ## A `..` range's count as a `..=` range's.
    static fn widen_count(n: SpanCount) -> FullCount

    ## A `..=` range's count as a `..` range's, which holds it.
    static fn narrow_count(n: FullCount) -> SpanCount

extend i8 implements Integer:
    type SpanCount = int
    type FullCount = int
    type Position = int

    static fn widen_count(n: int) -> int:
        return n

    static fn narrow_count(n: int) -> int:
        return n

    fn successor() -> i8:
        return this + 1

    fn predecessor() -> i8:
        return this - 1

    fn distance_to(end: i8) -> int:
        return (end as int) - (this as int)

    fn back_by(n: int) -> i8:
        return ((this as int) - n) as i8

    fn forward_by(n: int) -> i8:
        return ((this as int) + n) as i8

    fn count_to_top() -> int:
        return this.distance_to(i8.MAX) + 1

extend i16 implements Integer:
    type SpanCount = int
    type FullCount = int
    type Position = int

    static fn widen_count(n: int) -> int:
        return n

    static fn narrow_count(n: int) -> int:
        return n

    fn successor() -> i16:
        return this + 1

    fn predecessor() -> i16:
        return this - 1

    fn distance_to(end: i16) -> int:
        return (end as int) - (this as int)

    fn back_by(n: int) -> i16:
        return ((this as int) - n) as i16

    fn forward_by(n: int) -> i16:
        return ((this as int) + n) as i16

    fn count_to_top() -> int:
        return this.distance_to(i16.MAX) + 1

extend i32 implements Integer:
    type SpanCount = int
    type FullCount = int
    type Position = int

    static fn widen_count(n: int) -> int:
        return n

    static fn narrow_count(n: int) -> int:
        return n

    fn successor() -> i32:
        return this + 1

    fn predecessor() -> i32:
        return this - 1

    fn distance_to(end: i32) -> int:
        return (end as int) - (this as int)

    fn back_by(n: int) -> i32:
        return ((this as int) - n) as i32

    fn forward_by(n: int) -> i32:
        return ((this as int) + n) as i32

    fn count_to_top() -> int:
        return this.distance_to(i32.MAX) + 1

extend i64 implements Integer:
    type SpanCount = u64
    type FullCount = u128
    type Position = i128

    static fn widen_count(n: u64) -> u128:
        return n

    static fn narrow_count(n: u128) -> u64:
        return n as u64

    fn successor() -> i64:
        return this + 1

    fn predecessor() -> i64:
        return this - 1

    ## G8-4 — `end - this` in 64 unsigned bits wraps to the gap exactly, which is at most
    ## `u64.MAX`.
    @overflow(wrap)
    fn distance_to(end: i64) -> u64:
        return (end as u64) - (this as u64)

    @overflow(wrap)
    fn back_by(n: u64) -> i64:
        return ((this as u64) - n) as i64

    @overflow(wrap)
    fn forward_by(n: u64) -> i64:
        return ((this as u64) + n) as i64

    fn count_to_top() -> u128:
        return (this.distance_to(i64.MAX) as u128) + 1

extend i128 implements Integer:
    type SpanCount = u128
    type FullCount = u256
    type Position = i256

    static fn widen_count(n: u128) -> u256:
        return n

    static fn narrow_count(n: u256) -> u128:
        return n as u128

    fn successor() -> i128:
        return this + 1

    fn predecessor() -> i128:
        return this - 1

    ## G8-4 — `end - this` in 128 unsigned bits wraps to the gap exactly.
    @overflow(wrap)
    fn distance_to(end: i128) -> u128:
        return (end as u128) - (this as u128)

    @overflow(wrap)
    fn back_by(n: u128) -> i128:
        return ((this as u128) - n) as i128

    @overflow(wrap)
    fn forward_by(n: u128) -> i128:
        return ((this as u128) + n) as i128

    fn count_to_top() -> u256:
        return (this.distance_to(i128.MAX) as u256) + 1

extend isize implements Integer:
    type SpanCount = u64
    type FullCount = u128
    type Position = i128

    static fn widen_count(n: u64) -> u128:
        return n

    static fn narrow_count(n: u128) -> u64:
        return n as u64

    fn successor() -> isize:
        return this + 1

    fn predecessor() -> isize:
        return this - 1

    ## G8-4 — `end - this` in 64 unsigned bits wraps to the gap exactly, which is at most
    ## `u64.MAX`.
    @overflow(wrap)
    fn distance_to(end: isize) -> u64:
        return (end as u64) - (this as u64)

    @overflow(wrap)
    fn back_by(n: u64) -> isize:
        return ((this as u64) - n) as isize

    @overflow(wrap)
    fn forward_by(n: u64) -> isize:
        return ((this as u64) + n) as isize

    fn count_to_top() -> u128:
        return (this.distance_to(isize.MAX) as u128) + 1

extend u8 implements Integer:
    type SpanCount = int
    type FullCount = int
    type Position = int

    static fn widen_count(n: int) -> int:
        return n

    static fn narrow_count(n: int) -> int:
        return n

    fn successor() -> u8:
        return this + 1

    fn predecessor() -> u8:
        return this - 1

    fn distance_to(end: u8) -> int:
        return (end as int) - (this as int)

    fn back_by(n: int) -> u8:
        return ((this as int) - n) as u8

    fn forward_by(n: int) -> u8:
        return ((this as int) + n) as u8

    fn count_to_top() -> int:
        return this.distance_to(u8.MAX) + 1

extend u16 implements Integer:
    type SpanCount = int
    type FullCount = int
    type Position = int

    static fn widen_count(n: int) -> int:
        return n

    static fn narrow_count(n: int) -> int:
        return n

    fn successor() -> u16:
        return this + 1

    fn predecessor() -> u16:
        return this - 1

    fn distance_to(end: u16) -> int:
        return (end as int) - (this as int)

    fn back_by(n: int) -> u16:
        return ((this as int) - n) as u16

    fn forward_by(n: int) -> u16:
        return ((this as int) + n) as u16

    fn count_to_top() -> int:
        return this.distance_to(u16.MAX) + 1

extend u32 implements Integer:
    type SpanCount = int
    type FullCount = int
    type Position = int

    static fn widen_count(n: int) -> int:
        return n

    static fn narrow_count(n: int) -> int:
        return n

    fn successor() -> u32:
        return this + 1

    fn predecessor() -> u32:
        return this - 1

    fn distance_to(end: u32) -> int:
        return (end as int) - (this as int)

    fn back_by(n: int) -> u32:
        return ((this as int) - n) as u32

    fn forward_by(n: int) -> u32:
        return ((this as int) + n) as u32

    fn count_to_top() -> int:
        return this.distance_to(u32.MAX) + 1

extend u64 implements Integer:
    type SpanCount = u64
    type FullCount = u128
    type Position = i128

    static fn widen_count(n: u64) -> u128:
        return n

    static fn narrow_count(n: u128) -> u64:
        return n as u64

    fn successor() -> u64:
        return this + 1

    fn predecessor() -> u64:
        return this - 1

    fn distance_to(end: u64) -> u64:
        return (end - this)

    fn back_by(n: u64) -> u64:
        return this - n

    fn forward_by(n: u64) -> u64:
        return this + n

    fn count_to_top() -> u128:
        return (this.distance_to(u64.MAX) as u128) + 1

extend u128 implements Integer:
    type SpanCount = u128
    type FullCount = u256
    type Position = i256

    static fn widen_count(n: u128) -> u256:
        return n

    static fn narrow_count(n: u256) -> u128:
        return n as u128

    fn successor() -> u128:
        return this + 1

    fn predecessor() -> u128:
        return this - 1

    fn distance_to(end: u128) -> u128:
        return (end - this)

    fn back_by(n: u128) -> u128:
        return this - n

    fn forward_by(n: u128) -> u128:
        return this + n

    fn count_to_top() -> u256:
        return (this.distance_to(u128.MAX) as u256) + 1

extend usize implements Integer:
    type SpanCount = u64
    type FullCount = u128
    type Position = i128

    static fn widen_count(n: u64) -> u128:
        return n

    static fn narrow_count(n: u128) -> u64:
        return n as u64

    fn successor() -> usize:
        return this + 1

    fn predecessor() -> usize:
        return this - 1

    fn distance_to(end: usize) -> u64:
        return (end - this) as u64

    fn back_by(n: u64) -> usize:
        return this - (n as usize)

    fn forward_by(n: u64) -> usize:
        return this + (n as usize)

    fn count_to_top() -> u128:
        return (this.distance_to(usize.MAX) as u128) + 1

## `[STD-4]` — an integer that is not zero, made by `NonZero.new`. Since no
## `NonZero` holds 0, `Option[NonZero[T]]` stores `None` as 0 and is the size
## of `T` (`[TYP-13]`), and dividing by one needs no check for zero
## (`[EFF-16]`): `x // d` and `x % d` take a `NonZero` of `x`'s type.
@derive(Copy, Hash)
[T: needs Integer]
pub struct NonZero:
    value: T

    ## `v`, or `None` when it is 0.
    pub static fn new(v: T) -> Option[NonZero[T]]:
        if v == T.default():
            return None
        return Some(NonZero(v))

    ## The integer.
    pub fn get() -> T:
        return this.value

extend i8 implements FloorDiv[NonZero[i8]], Rem[NonZero[i8]]:
    type Output = i8

    fn floordiv(d: NonZero[i8]) -> i8:
        return this // d.value

    fn rem(d: NonZero[i8]) -> i8:
        return this % d.value

extend i16 implements FloorDiv[NonZero[i16]], Rem[NonZero[i16]]:
    type Output = i16

    fn floordiv(d: NonZero[i16]) -> i16:
        return this // d.value

    fn rem(d: NonZero[i16]) -> i16:
        return this % d.value

extend i32 implements FloorDiv[NonZero[i32]], Rem[NonZero[i32]]:
    type Output = i32

    fn floordiv(d: NonZero[i32]) -> i32:
        return this // d.value

    fn rem(d: NonZero[i32]) -> i32:
        return this % d.value

extend i64 implements FloorDiv[NonZero[i64]], Rem[NonZero[i64]]:
    type Output = i64

    fn floordiv(d: NonZero[i64]) -> i64:
        return this // d.value

    fn rem(d: NonZero[i64]) -> i64:
        return this % d.value

extend i128 implements FloorDiv[NonZero[i128]], Rem[NonZero[i128]]:
    type Output = i128

    fn floordiv(d: NonZero[i128]) -> i128:
        return this // d.value

    fn rem(d: NonZero[i128]) -> i128:
        return this % d.value

extend isize implements FloorDiv[NonZero[isize]], Rem[NonZero[isize]]:
    type Output = isize

    fn floordiv(d: NonZero[isize]) -> isize:
        return this // d.value

    fn rem(d: NonZero[isize]) -> isize:
        return this % d.value

extend u8 implements FloorDiv[NonZero[u8]], Rem[NonZero[u8]]:
    type Output = u8

    fn floordiv(d: NonZero[u8]) -> u8:
        return this // d.value

    fn rem(d: NonZero[u8]) -> u8:
        return this % d.value

extend u16 implements FloorDiv[NonZero[u16]], Rem[NonZero[u16]]:
    type Output = u16

    fn floordiv(d: NonZero[u16]) -> u16:
        return this // d.value

    fn rem(d: NonZero[u16]) -> u16:
        return this % d.value

extend u32 implements FloorDiv[NonZero[u32]], Rem[NonZero[u32]]:
    type Output = u32

    fn floordiv(d: NonZero[u32]) -> u32:
        return this // d.value

    fn rem(d: NonZero[u32]) -> u32:
        return this % d.value

extend u64 implements FloorDiv[NonZero[u64]], Rem[NonZero[u64]]:
    type Output = u64

    fn floordiv(d: NonZero[u64]) -> u64:
        return this // d.value

    fn rem(d: NonZero[u64]) -> u64:
        return this % d.value

extend u128 implements FloorDiv[NonZero[u128]], Rem[NonZero[u128]]:
    type Output = u128

    fn floordiv(d: NonZero[u128]) -> u128:
        return this // d.value

    fn rem(d: NonZero[u128]) -> u128:
        return this % d.value

extend usize implements FloorDiv[NonZero[usize]], Rem[NonZero[usize]]:
    type Output = usize

    fn floordiv(d: NonZero[usize]) -> usize:
        return this // d.value

    fn rem(d: NonZero[usize]) -> usize:
        return this % d.value
