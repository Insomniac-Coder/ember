#$ test: compile-fail
#$ rules: TYP-24
## `[TYP-24]` - inside a generic body a method that two of a type parameter's bounds both declare is `E2070`,
## whose message names both bounds and the call to write.

interface Shape:
    fn area() -> int

interface Drawable:
    fn area() -> int

[T: needs (Shape, Drawable)]
fn measure(x: T) -> int:
    return x.area()                  #$ error[E2070]: `area` is offered by both `Shape` and `Drawable`: write `Shape.area(x)` or `Drawable.area(x)`

[T: needs (Shape, Drawable)]
fn measure_named(x: T) -> int:
    return Shape.area(x) + Drawable.area(x)

fn main():
    println(1)
