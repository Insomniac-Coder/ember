#$ test: compile-fail
#$ rules: TYP-17, IFC-4
# ODR-040 — with `T: Add`, `a + b` is a `T.Output`, which only a signature
# naming it can hold; a bound's binding is checked at each call.

[T: needs Add]
fn bare(a: T, b: T) -> T:
    return a + b    #$ error[E2040]: `add` on `T` returns a `T.Output`, and nothing here says what that is

[T: needs Add]
fn named(a: T, b: T) -> T.Output:
    return a + b

[T: needs (Add[Output = T], Default)]
fn sum(xs: Span[T]) -> T:
    total = T.default()
    for x in xs:
        total = total + x
    return total

@derive(Copy)
struct R:
    x: int

extend R implements Add, Default:
    type Output = int

    fn add(rhs: R) -> int:
        return this.x + rhs.x

    static fn default() -> R:
        return R(x=0)

fn main():
    println(named(1, 2), named(R(x=1), R(x=2)))
    rs: Array[R] = [R(x=1)]
    println(sum(rs.as_span()))    #$ error[E2040]: `R`'s `Output` for `Add[R]` is `i64`, but `T`'s bound needs `R`
