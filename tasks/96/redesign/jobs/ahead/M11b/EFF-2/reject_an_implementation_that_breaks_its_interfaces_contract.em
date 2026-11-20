# Clause EFF-2.2: "An interface method may declare contracts; its implementations MUST satisfy them
# (`E4010`)."
#$ test: compile-fail
#$ rules: EFF-2, EFF-6
# `Circle.area` allocates, and `Shape.area` is `@noalloc`. No text in the spec: code only. Where
# the refusal stands (the implementing method) is the reading this file takes.

interface Shape:
    @noalloc
    fn area() -> float

struct Square implements Shape:
    side: float

    fn area() -> float:
        return this.side * this.side

struct Circle implements Shape:
    r: float

    fn area() -> float:    #$ error[E4010]
        cache = [this.r]
        return cache[0] * cache[0] * 3.0

fn main():
    println(Square(2.0).area())
    println(Circle(1.0).area())
