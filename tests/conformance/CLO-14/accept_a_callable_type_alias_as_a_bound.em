#$ test: run-pass
#$ rules: CLO-14, TYP-17
#$ stdout: 6 3
# D-425 — a type alias of a callable type is a callable bound, as it is a
# parameter's type: `type Op = fn(int) -> int` then `F: Op`. It was taken
# for an interface and refused, `E1010` "cannot find interface `Op`".

type Op = fn(int) -> int

fn h[F: Op](f: F, x: int) -> int:
    return f(x)

fn k(f: Op, x: int) -> int:
    return f(x)

fn main():
    println(h(fn(v) => v * 3, 2), k(fn(v) => v + 1, 2))
