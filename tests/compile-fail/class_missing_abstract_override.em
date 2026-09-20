#$ test: compile-fail
#$ rules: CLS-4
#$ error[E2020]: concrete class `Square` does not implement abstract method `area`

abstract class Shape:
    virtual fn area(self) -> i32

    fn init(mut self):
        pass

class Square(Shape):
    fn init(mut self):
        super.init()

fn main():
    square = Square()
    println(1)
