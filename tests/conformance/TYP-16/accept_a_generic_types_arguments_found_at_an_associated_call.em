#$ test: run-pass
#$ rules: TYP-16, TYP-18
#$ stdout: 5 5
#$ stdout: 2.5 x
# `[TYP-16]` — `Pair.make(5)` calls an associated function of a generic
# struct named without its arguments; they are found as a generic function's
# are, from the arguments (a literal taking its default type) and from the
# type the context expects (D-334: this was `E2020`).

struct Pair[T]:
    a: T
    b: T

    fn make(v: T) -> Pair[T]:
        return Pair(v, v)

    fn with_second(first: T, second: T) -> Self:
        return Pair(first, second)

fn main():
    p = Pair.make(5)
    println(p.a, p.b)
    q: Pair[f32] = Pair.make(2.5)
    r = Pair.with_second('w', 'x')
    println(q.a, r.b)
