#$ test: compile-fail
#$ rules: TYP-17, IFC-4
# ODR-040 — with `T: Add`, `a + b` is a `T.Output`, which only a signature
# naming it can hold; a bound's binding is checked at each call.

fn bare[T: Add](a: T, b: T) -> T:
    return a + b    #$ error[E2040]: `add` on `T` returns a `T.Output`, and nothing here says what that is

fn named[T: Add](a: T, b: T) -> T.Output:
    return a + b

fn sum[T: Add[Output = T] + Default](xs: Span[T]) -> T:
    total = T.default()
    for x in xs:
        total = total + x
    return total

@derive(Copy)
struct R:
    x: int

extend R implements Add, Default:
    type Output = int

    fn add(self, rhs: R) -> int:
        return self.x + rhs.x

    fn default() -> R:
        return R(x=0)

fn main():
    println(named(1, 2), named(R(x=1), R(x=2)))
    rs: Array[R] = [R(x=1)]
    println(sum(rs.as_span()))    #$ error[E2040]: `R`'s `Output` for `std.core.Add[R]` is `i64`, but `T`'s bound needs `R`
