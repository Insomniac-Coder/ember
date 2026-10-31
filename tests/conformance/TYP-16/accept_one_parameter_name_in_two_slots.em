#$ test: run-pass
#$ rules: TYP-16, CLO-3
#$ stdout: Some(4) Some(6) 14
# D-306 — a type over a type parameter is one type per slot: `Option[U]`
# where `U` is a function's first parameter is not `Option[U]` where `U` is
# a method's second (std's `Option.map[U]`). Instance names held only the
# parameter's name, so the first `Option[U]` made stood for every later
# one, and a callable parameter's slot took the result's type: these
# printed "not implemented" errors for an `Option` of a function type.

[U]
fn wrap(x: int, f: fn(ref int) -> U) -> Option[U]:
    return Some(f(x))

struct B:
    items: Array[int]

    [U]
    fn first_as(f: fn(ref int) -> U) -> Option[U]:
        for v in this.items:
            return Some(f(v))
        return None

[T]
struct G:
    items: Array[T]

    [U]
    fn apply(f: fn(ref T) -> U) -> U:
        return f(this.items[0])

fn main():
    b = B([5])
    g = G[int]([7])
    println(wrap(3, fn(v) => v + 1), b.first_as(fn(v) => v + 1), g.apply(fn(v) => v * 2))
