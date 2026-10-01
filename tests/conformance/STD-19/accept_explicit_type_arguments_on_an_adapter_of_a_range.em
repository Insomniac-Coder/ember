#$ test: run-pass
#$ rules: STD-19, TYP-18
#$ stdout: 6 9 3.0
# D-427 — `E2060`'s help says to write a method's type argument out
# (`.map[T](...)`), and a range's adapters take it as an iterator's do:
# `(0..3).map[int](f)` was `E1010` ("`Range[i64]` has no method named
# `map`") where `(0..3).map(f)` resolves. The argument is the call's:
# `map[f64]` makes the lambda's `1` an `f64`.

fn twice(x: int) -> int:
    return x * 2

fn main():
    total = (0..3).map[int](twice).sum()
    xs: Array[int] = [1, 2, 3]
    other = xs.iter().map[int](fn(v: int) => v + 1).sum()
    ones = (0..3).map[f64](fn(x: int) => 1).sum()
    println(total, other, ones)
