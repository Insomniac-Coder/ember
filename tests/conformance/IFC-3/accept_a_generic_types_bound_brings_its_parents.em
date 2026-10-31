#$ test: run-pass
#$ rules: IFC-3, TYP-17
#$ stdout: 0 0
#$ stdout: true false
# `[IFC-3]` — a bound brings its parents, on a generic type's parameter as on
# a generic function's: with `interface Whole: Default + Eq`, the methods of
# `Tally[T: Whole]` may call `T.default()` and compare with `==` (D-335: a
# generic type's parameters were declared before any interface had parents
# to bring).

interface Whole: needs (Default, Eq):
    pass

extend i32 implements Whole:
    pass

[T: needs Whole]
fn zero_of(v: T) -> T:
    return T.default()

[T: needs Whole]
struct Tally:
    v: T

    fn zero() -> T:
        return T.default()

    fn is_zero() -> bool:
        return this.v == T.default()

fn main():
    five: i32 = 5
    t = Tally(five)
    println(zero_of(five), t.zero())
    empty: i32 = 0
    println(Tally(empty).is_zero(), Tally(five).is_zero())
