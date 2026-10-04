#$ test: run-pass
#$ rules: TYP-16, CLS-4, DSP-2
#$ profiles: debug, release, shipping
#$ assert-c: contains(static int32_t em_vt_Square_bool_slot0)
#$ assert-c: contains(->slot0)
#$ stdout: 42
#$ stdout: 7
# Two concrete classes implement `area`, so a call through `Shape[bool]` reads
# the object's table ([DSP-2]); with one, it would be a direct call ([DSP-5]).

abstract class Shape[T]:
    virtual fn area(self) -> i32

    fn init(mut self):
        pass

class Square[T](Shape[T]):
    fn init(mut self):
        super.init()

    override fn area(self) -> i32:
        return 42

class Circle[T](Shape[T]):
    fn init(mut self):
        super.init()

    override fn area(self) -> i32:
        return 7

fn main():
    square = Square[bool]()
    shape: Shape[bool] = square
    println(shape.area())
    other: Shape[bool] = Circle[bool]()
    println(other.area())
