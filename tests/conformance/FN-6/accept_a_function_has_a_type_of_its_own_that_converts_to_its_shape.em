#$ test: run-pass
#$ rules: FN-6, CLO-1, CLO-3, TYP-5, TYP-23
#$ profiles: debug, development, release
#$ stdout: 2 -2
#$ 2 3 18
#$ 6 11 12
#$ 2 0 1 2
#$ 12 7
#$ true false false true
# ODR-278, `[FN-6]`, `[CLO-1]` — every function and every lambda that captures nothing has a type
# of its own, which converts to its shape `fn(A) -> R` wherever a value of that shape is wanted:
# a written callable parameter, a field, a list, a comparison with a callable value. Where two
# functions meet — a conditional, an assignment in each branch, a second assignment, a generic
# parameter given both — the place holds their shape. A generic given a function is made for
# that function (`twice(add_one, 1)`).

fn add_one(x: int) -> int:
    return x + 1

fn sub_one(x: int) -> int:
    return x - 1

fn pick(c: bool) -> int:
    if c:
        f = add_one
    else:
        f = sub_one
    g = add_one
    g = sub_one
    h = add_one if c else sub_one
    return f(0) + g(2) + h(0) - 1

fn apply(f: fn(int) -> int, x: int) -> int:
    return f(x)

fn twice[F: fn(int) -> int](f: F, x: int) -> int:
    return f(f(x))

fn either[T](a: T, b: T, first: bool) -> T:
    return a if first else b

struct Holder:
    f: fn(int) -> int

fn main():
    println(pick(true), pick(false))
    println(apply(add_one, 1), twice(add_one, 1), twice(fn(x) => x * 3, 2))
    hs = Holder(add_one)
    k = fn(x: int) -> int => x + 10
    println(hs.f(5), k(1), apply(k, 2))
    fs = [add_one, sub_one]
    (n, m) = (add_one, 1)
    (n, m) = (sub_one, 2)
    println(fs[0](1), fs[1](1), n(m), either(add_one, sub_one, true)(1))
    xs = [1, 2, 3]
    println(xs.iter().map(fn(x: ref int) -> int => x * 2).sum(), xs.iter().map(add_one).sum() - 2)
    same: fn(int) -> int = add_one
    f2 = add_one
    g2 = sub_one
    println(same == add_one, same == sub_one, f2 == g2, f2 == add_one)
