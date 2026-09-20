#$ test: compile-fail
#$ rules: CLS-4
#$ error[E2020]: concrete class `Square_bool` does not implement abstract method `area`

abstract class Shape[T]:
    virtual fn area(self) -> i32

    fn init(mut self):
        pass

class Square[T](Shape[T]):
    fn init(mut self):
        super.init()

fn main():
    square = Square[bool]()
    println(1)
