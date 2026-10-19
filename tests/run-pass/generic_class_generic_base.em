#$ test: run-pass
#$ rules: TYP-16, CLS-2, CLS-4
#$ profiles: debug, development, release
#$ stdout: 42

open class Base[T]:
    value: T

    fn init(mut self, value: T):
        self.value = value

class Derived[T](Base[T]):
    marker: bool

    fn init(mut self, value: T, marker: bool):
        self.marker = marker
        super.init(value)

fn main():
    item = Derived[i32](42, false)
    println(item.value)
