#$ test: run-pass
#$ rules: CLS-4, DSP-2
#$ profiles: debug, release, shipping
#$ assert-c: contains(static int32_t em_vt_Square_slot0)
#$ assert-c: contains(->slot0)
#$ stdout: 42

abstract class Shape:
    virtual fn area(self) -> i32

    fn init(mut self):
        pass

class Square(Shape):
    fn init(mut self):
        super.init()

    override fn area(self) -> i32:
        return 42

fn main():
    square = Square()
    shape: Shape = square
    println(shape.area())
