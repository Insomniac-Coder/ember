#$ test: run-pass
#$ rules: TYP-16, CLO-3
#$ stdout: Some(4) Some(6) 14
# D-306 — a type over a type parameter is one type per slot: `Option[U]`
# where `U` is a function's first parameter is not `Option[U]` where `U` is
# a method's second (std's `Option.map[U]`). Instance names held only the
# parameter's name, so the first `Option[U]` made stood for every later
# one, and a callable parameter's slot took the result's type: these
# printed "not implemented" errors for an `Option` of a function type.

fn wrap[U](x: int, f: fn(ref int) -> U) -> Option[U]:
    return Some(f(x))

struct B:
    items: Array[int]

    fn first_as[U](self, f: fn(ref int) -> U) -> Option[U]:
        for v in self.items:
            return Some(f(v))
        return None

struct G[T]:
    items: Array[T]

    fn apply[U](self, f: fn(ref T) -> U) -> U:
        return f(self.items[0])

fn main():
    b = B([5])
    g = G[int]([7])
    println(wrap(3, fn(v) => v + 1), b.first_as(fn(v) => v + 1), g.apply(fn(v) => v * 2))
