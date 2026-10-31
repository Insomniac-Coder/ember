#$ test: run-pass
#$ rules: TYP-16, CLS-4, DSP-2
#$ profiles: debug, development, release
#$ assert-c: contains(static int32_t em_vt_6SquareGda5cc12a188c7619_slot0)
#$ assert-c: contains(->slot0)
#$ stdout: 42
#$ stdout: 7
# Two concrete classes implement `area`, so a call through `Shape[bool]` reads
# the object's table ([DSP-2]); with one, it would be a direct call ([DSP-5]).

[T]
abstract class Shape:
    virtual fn area() -> i32

    fn init():
        pass

[T]
class Square(Shape[T]):
    fn init():
        super.init()

    override fn area() -> i32:
        return 42

[T]
class Circle(Shape[T]):
    fn init():
        super.init()

    override fn area() -> i32:
        return 7

fn main():
    square = Square[bool]()
    shape: Shape[bool] = square
    println(shape.area())
    other: Shape[bool] = Circle[bool]()
    println(other.area())
