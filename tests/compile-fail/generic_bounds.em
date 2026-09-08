#$ test: compile-fail
#$ rules: TYP-17, TYP-18
#$ error[E2040]: `T` has no method `area`; its bounds do not provide one
#$ error[E2040]: `Plain` does not implement `Shape`

interface Shape:
    fn area(self) -> f32

struct Square implements Shape:
    side: f32
    fn area(self) -> f32:
        return self.side * self.side

struct Plain:
    x: i32

# `[TYP-17]` — no bound, so `area` is not available here. There is no duck
# typing, even though every caller happens to pass something that has one.
fn unbounded[T](a: T) -> f32:
    return a.area()

fn bounded[T: Shape](a: T) -> f32:
    return a.area()

fn main():
    println(unbounded(Square(2.0)))
    # `[TYP-17]` — the bound is not satisfied by this argument.
    println(bounded(Plain(1)))
