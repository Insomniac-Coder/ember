#$ test: compile-fail
#$ rules: TYP-17, TYP-18
#$ error[E2040]: `T` has no method `area`; its bounds do not provide one
#$ error[E2040]: `Plain` does not implement `Shape`

interface Shape:
    fn area() -> f32

struct Square implements Shape:
    side: f32
    fn area() -> f32:
        return this.side * this.side

struct Plain:
    x: i32

# `[TYP-17]` — no bound, so `area` is not available here. There is no duck
# typing, even though every caller happens to pass something that has one.
[T]
fn unbounded(a: T) -> f32:
    return a.area()

[T: needs Shape]
fn bounded(a: T) -> f32:
    return a.area()

fn main():
    println(unbounded(Square(2.0)))
    # `[TYP-17]` — the bound is not satisfied by this argument.
    println(bounded(Plain(1)))
