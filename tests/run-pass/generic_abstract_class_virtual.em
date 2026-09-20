#$ test: run-pass
#$ rules: TYP-16, CLS-4, DSP-2
#$ profiles: debug, release, shipping
#$ assert-c: contains(static int32_t em_vt_Square_bool_slot0)
#$ assert-c: contains(->slot0)
#$ stdout: 42

abstract class Shape[T]:
    virtual fn area(self) -> i32

    fn init(mut self):
        pass

class Square[T](Shape[T]):
    fn init(mut self):
        super.init()

    override fn area(self) -> i32:
        return 42

fn main():
    square = Square[bool]()
    shape: Shape[bool] = square
    println(shape.area())
