#$ test: run-pass
#$ rules: TYP-37, TYP-9
#$ stdout: false false
#$ false false false false
#$ -0.0 0.0 1.0
#$ -0.0 0.0
#$ Ok(1) true
# `[TYP-37]` (ODR-055, SP-018) — a float's comparison operators are IEEE in
# generic code as in concrete code: extracting `a < b` into a function bounded
# by `Ord` does not turn it into a total order. `sort()`, `min`, `max` and
# `binary_search` use `cmp` (total: `-0.0` before `0.0`); `contains` uses
# `==` (so `-0.0` is found where `0.0` is).

fn less[T: Ord](a: T, b: T) -> bool:
    return a < b

fn main():
    nan = 0.0 / 0.0
    println(-0.0 < 0.0, less(-0.0, 0.0))
    println(nan < 1.0, less(nan, 1.0), 1.0 < nan, less(1.0, nan))
    xs = [1.0, 0.0, -0.0]
    xs.sort()
    println(xs[0], xs[1], xs[2])
    println(min(0.0, -0.0), max(-0.0, 0.0))
    ys = [-0.0, 0.0, 1.0]
    println(ys.binary_search(0.0), [0.0].contains(-0.0))
