#$ test: run-pass
#$ rules: CLO-3, CLS-1
#$ stdout: -2 15 6
# `[CLO-3]` — a callable parameter of `init` is an implicit generic like any
# other callable parameter: constructing the class instantiates `init` for
# the argument, a function or a lambda, capturing or not (D-235).

fn neg(x: int) -> int:
    return -x

class B:
    n: int

    fn init(mut self, f: fn(int) -> int, x: int):
        self.n = f(x)

fn main():
    k = 10
    a = B(neg, 2)
    b = B(fn(v) => v + k, 5)
    c = B(fn(v: int) => v * 3, 2)
    println(a.n, b.n, c.n)
