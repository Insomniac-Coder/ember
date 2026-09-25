#$ test: run-pass
#$ rules: TYP-21, TYP-23, IFC-4
#$ profiles: debug, release, shipping
#$ stdout: 500 500 750 1250
#$ 6.0 6.0
# `[TYP-21]` — `k * m` with a number on the left is the number type's
# `Mul[Money]`, which a program may implement as it implements `Money`'s own
# `Mul[i64]`. An untyped literal on the left takes the number type that
# implements the operator for the right operand (`[TYP-23]`), as it takes
# the type a parameter wants: `2 * m` is `i64`'s. Between two numbers the
# operator stays the built-in one.

@derive(Copy)
struct Money:
    cents: i64

extend Money implements Mul[i64]:
    type Output = Money

    fn mul(self, k: i64) -> Money:
        return Money(self.cents * k)

extend i64 implements Mul[Money]:
    type Output = Money

    fn mul(self, m: Money) -> Money:
        return m * self

fn main():
    m = Money(250)
    k: i64 = 3
    println((m * 2).cents, (2 * m).cents, (k * m).cents, (5 * m).cents)
    x: f64 = 2.0
    println(x * 3.0, 3.0 * x)
