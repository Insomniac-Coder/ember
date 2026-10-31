#$ test: run-pass
#$ rules: CLS-4, DSP-2
#$ profiles: debug, development, release
#$ assert-c: contains(static int32_t em_vt_6Square_slot0)
#$ assert-c: contains(->slot0)
#$ stdout: 42
#$ stdout: 7
# Two concrete classes implement `area`, so a call through `Shape` reads the
# object's table ([DSP-2]); with one, it would be a direct call ([DSP-5]).

abstract class Shape:
    virtual fn area() -> i32

    fn init():
        pass

class Square(Shape):
    fn init():
        super.init()

    override fn area() -> i32:
        return 42

class Circle(Shape):
    fn init():
        super.init()

    override fn area() -> i32:
        return 7

fn main():
    square = Square()
    shape: Shape = square
    println(shape.area())
    other: Shape = Circle()
    println(other.area())
