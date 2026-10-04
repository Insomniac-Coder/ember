#$ test: run-pass
#$ rules: DSP-5, DSP-2, CLS-4
#$ profiles: debug, release, shipping
#$ stdout: 1 0
#$ stdout: 4 0
#$ stdout: 4 4
#$ assert-c-count: contains("->ti->vtable))->slot") == 1
# `[DSP-5]` — the whole program is in the build, so a virtual call that only
# one body can answer is a direct call: `sides`, which nothing overrides, and
# `legs`, whose abstract class has one concrete class below it. `name` has two
# bodies and is still read from the object's table (`[DSP-2]`). The milestone
# `the_optimization_report_lists_each_direct_call` reads this program's report.

open class Shape:
    virtual fn sides(self) -> int:
        return 0

    virtual fn name(self) -> int:
        return 1

    fn init(mut self):
        pass

class Square(Shape):
    override fn name(self) -> int:
        return 4

    fn init(mut self):
        super.init()

abstract class Animal:
    virtual fn legs(self) -> int

    fn init(mut self):
        pass

class Dog(Animal):
    override fn legs(self) -> int:
        return 4

    fn init(mut self):
        super.init()

fn describe(s: Shape) -> (int, int):
    return (s.name(), s.sides())

fn count_legs(a: Animal) -> int:
    return a.legs()

fn main():
    plain: Shape = Shape()
    square: Shape = Square()
    p = describe(plain)
    q = describe(square)
    println(p.0, p.1)
    println(q.0, q.1)
    dog: Animal = Dog()
    println(count_legs(dog), Dog().legs())
