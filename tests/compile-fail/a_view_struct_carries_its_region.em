#$ test: compile-fail
#$ rules: LT-2, LT-1a, LT-5, TYP-14
#$ error[E3060]: `n` does not live long enough
#$ error[E3062]: the returned view points into `b`, which `@borrows` does not name
#$ error[E3021]: `k` cannot be written while it is borrowed

## `[LT-2]` — a view struct has one region, and a view-typed field's region is
## the struct's. So a `Cursor` is exactly as long-lived as what it points at,
## and everything `[LT-1]` says about a returned reference it says about a
## returned `Cursor`.

@view
struct Cursor:
    at: ref i32

## The borrow is of a local, so the struct that carries it may not leave.
fn escaping() -> Cursor:
    n: i32 = 1
    c: Cursor = Cursor(ref n)
    return c

## `[LT-1a]` reaches through the struct: the returned view points into `b`
## whatever it is wrapped in.
@borrows(a)
fn wrapped(a: ref i32, b: ref i32) -> Cursor:
    return Cursor(b)

## And the caller holds the borrow for as long as it holds the struct.
fn hand_back(a: ref i32) -> Cursor:
    return Cursor(a)

fn conflicting() -> i32:
    k: i32 = 1
    c: Cursor = hand_back(ref k)
    k = 5
    return c.at

fn main():
    println(escaping().at)
    x: i32 = 1
    y: i32 = 2
    println(wrapped(ref x, ref y).at)
    println(conflicting())
