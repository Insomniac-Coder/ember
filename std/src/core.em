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
