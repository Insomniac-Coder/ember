#$ test: compile-fail
#$ rules: CLS-4
#$ error[E2020]: concrete class `Square[bool]` does not implement abstract method `area`

[T]
abstract class Shape:
    virtual fn area() -> i32

    fn init():
        pass

[T]
class Square(Shape[T]):
    fn init():
        super.init()

fn main():
    square = Square[bool]()
    println(1)
