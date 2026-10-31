#$ test: compile-fail
#$ rules: CLS-4
#$ error[E2020]: concrete class `Square` does not implement abstract method `area`

abstract class Shape:
    virtual fn area() -> i32

    fn init():
        pass

class Square(Shape):
    fn init():
        super.init()

fn main():
    square = Square()
    println(1)
