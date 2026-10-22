#$ test: run-pass
#$ rules: MOD-5, TYP-1
#$ profiles: debug, development, release
#$ stdout: 3 4 hi
#$ 1 2
#$ 5 7
# `[MOD-5]` — a local declaration shadows a prelude name, and `int`, `float` and `str` are
# prelude names (`[TYP-1]`: aliases of `i64` and `f64`; text): a program's own `struct int` is
# what `int` means in its types, its constructions and its constants, and an integer literal is
# still an `i64` (D-860: in a type position the alias won, so `x: int` was an `i64` with no field
# `n`). D-305 did this for `Box` and `Cell`.

struct int:
    n: i64

    const ONE: i64 = 1

struct float:
    whole: i64

struct str:
    text: String

fn get(x: int) -> i64:
    return x.n

fn wide(f: float) -> i64:
    return f.whole

fn main():
    v: int = int(3)
    s = str("hi".to_string())
    println(get(v), wide(float(4)), s.text)
    xs: Array[int] = [int(2)]
    println(int.ONE, xs[0].n)
    plain = 5
    total: i64 = plain + 2
    println(plain, total)
