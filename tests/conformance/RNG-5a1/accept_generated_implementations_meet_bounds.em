#$ test: run-pass
#$ rules: RNG-5a1, RNG-5, TYP-17, IFC-4
#$ stdout: 0.5 0.75 1.25 -0.25 -3 6
#$ stdout: 0.75 1.0 6 {3: 1}
# N1 (ADR-089) — a range type `T` over `R` has the generated `Op[T]` and
# `Op[R]`, and `R` has `Op[T]`, for each operator `R` has; `Neg` too; each
# with `Output = R`. Its `Ord` is `R`'s. So a bound asks for them, `T.Output`
# is `R`, and the method form is the operator. Each bound here was `E2040`,
# and `r.add(x)` `E1010`.

type Unit = f64 in 0.0 ..= 1.0
type Level = i32 in 0 ..= 10

fn twice[T: Add[T]](x: T) -> T.Output:
    return x + x

fn plus_repr[T: Add[f64]](x: T) -> T.Output:
    return x + 0.5

fn repr_plus[R: Add[Unit]](r: R, u: Unit) -> R.Output:
    return r + u

fn negate[T: Neg](x: T) -> T.Output:
    return -x

fn biggest[T: Ord + Copy](a: T, b: T) -> T:
    if a > b:
        return a
    return b

fn main():
    a = Unit.clamped(0.25)
    b = Unit.clamped(0.75)
    l = Level.clamped(3)
    println(twice(a), plus_repr(a), repr_plus(1.0, a), negate(a), negate(l), twice(l))
    counts: Map[Level, int] = {}
    counts[l] = 1
    println(biggest(a, b), a.add(b), l.mul(2), counts)
