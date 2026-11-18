#$ test: run-pass
#$ rules: TYP-9
#$ profiles: debug, development, release
#$ stdout: false true false false false false
#$ true false false true false true
#$ 0.0
## `[TYP-9]` - `NaN == NaN` is false and the six comparisons are IEEE's; a lambda takes the float mode of the
## function it is written in, so one in a strict function keeps `a * b + c` as two roundings.

fn probe(a: f64, b: f64) -> (bool, bool, bool, bool, bool, bool):
    return (a == b, a != b, a < b, a <= b, a > b, a >= b)

fn two_roundings() -> f32:
    a: f32 = 1.0 + 1.0 / 4096.0
    c: f32 = -(1.0 + 1.0 / 2048.0)
    f = fn(x: f32) => x * x + c
    return f(a)

fn main():
    nan = 0.0 / 0.0
    t = probe(nan, nan)
    one = probe(1.0, 1.0)
    println(t.0, t.1, t.2, t.3, t.4, t.5)
    println(one.0, one.1, one.2, one.3, one.4, one.5)
    println(two_roundings())
