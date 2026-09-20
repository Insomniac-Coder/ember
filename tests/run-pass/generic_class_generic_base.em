#$ test: run-pass
#$ rules: TYP-16, CLS-2, CLS-4
#$ profiles: debug, release, shipping
#$ stdout: 42

open class Base[T]:
    value: T

    fn init(mut self, value: T):
        self.value = value

class Derived[T](Base[T]):
    marker: bool

    fn init(mut self, value: T, marker: bool):
        super.init(value)
        self.marker = marker

fn main():
    item = Derived[i32](42, false)
    println(item.value)
