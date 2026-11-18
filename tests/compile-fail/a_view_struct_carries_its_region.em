#$ test: compile-fail
#$ rules: LT-2, LT-1a, LT-5, TYP-14
#$ error[E3060]: `n` does not live long enough
#$ error[E3062]: `wrapped` returns a view of `b`, but its first line says `from a`
#$ error[E3021]: `k` cannot be written while it is borrowed

## `[LT-2]` — a view struct has one region, and a view-typed field's region is
## the struct's. So a `Cursor` is exactly as long-lived as what it points at,
## and everything `[LT-1]` says about a returned reference it says about a
## returned `Cursor`.

@view
struct Cursor:
    at: ref String

## The borrow is of a local, so the struct that carries it may not leave.
fn escaping() -> Cursor:
    n: String = "1".to_string()
    c: Cursor = Cursor(ref n)
    return c

## `[LT-1a]` reaches through the struct: the returned view points into `b`
## whatever it is wrapped in.
fn wrapped(ref a: String, ref b: String) -> Cursor from a:
    return Cursor(b)

## And the caller holds the borrow for as long as it holds the struct.
fn hand_back(ref a: String) -> Cursor:
    return Cursor(a)

fn conflicting() -> int:
    k: String = "1".to_string()
    c: Cursor = hand_back(ref k)
    k = "5".to_string()
    return c.at.len()

fn main():
    println(escaping().at)
    x: String = "1".to_string()
    y: String = "2".to_string()
    println(wrapped(ref x, ref y).at)
    println(conflicting())
