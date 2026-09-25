#$ test: compile-fail
#$ rules: TYP-34
# `[TYP-34]` — "`@view` on a type that is not a view is `E2030`": it would
# document a restriction the type does not have.

@view  #$ error[E2030]: `@view` on `Point`, which is not a view type
struct Point:
    x: i32
    y: i32

@view  #$ error[E2030]
enum Mode:
    Fast
    Slow(level: int)

fn main():
    println(Point(1, 2).x)
