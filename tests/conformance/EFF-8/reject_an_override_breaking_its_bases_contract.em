#$ test: compile-fail
#$ rules: EFF-8
#$ error[E4041]: an override inherits its base method's contracts
# `[EFF-8]` — an `override` inherits its base method's contracts: `Shape.area` is `@noio`, so
# `Loud.area`, which prints, breaks it without writing it.

open class Shape:
    @noio
    virtual fn area(self) -> int:
        return 0

class Loud(Shape):
    override fn area(self) -> int:
        println("loud")
        return 1

fn main():
    s: Shape = Loud()
    println(s.area())
