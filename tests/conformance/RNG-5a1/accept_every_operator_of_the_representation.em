#$ test: run-pass
#$ rules: RNG-5a1, RNG-5, TYP-21
#$ stdout:
#$ 7 0 7 12 1 -4 9
#$ 7 -4 81 0.25
# D-420, ODR-095 — a range type has every operator its representation
# implements over itself, each the representation's: the bitwise operators,
# `~` and `**` as the operator form and through a bound. The bounds refused
# them (`T: BitOr[T]`, `T: Not`, `T: Pow[T]` were `E2040`) and `**` was
# refused in both forms, while `|` and `~` worked.

type Level = i32 in 0 ..= 10
type Unit = f64 in 0.0 ..= 1.0

fn either[T: BitOr[T]](a: T, b: T) -> T.Output:
    return a | b

fn flip[T: Not](a: T) -> T.Output:
    return ~a

fn power[T: Pow[T]](a: T, b: T) -> T.Output:
    return a ** b

fn main():
    l: Level = 3
    m: Level = 4
    println(l | m, l & m, l ^ m, l << 2, l >> 1, ~l, l ** 2)
    u = Unit.clamped(0.5)
    println(either(l, m), flip(l), power(l, m), u ** 2.0)
