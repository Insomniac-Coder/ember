#$ test: run-pass
#$ rules: TYP-16, CLS-2, CLS-4, CLS-6, DRP-1
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ stdout: base

open class Base[T]:
    value: i32
    marker: T

    fn init(mut self, value: i32, marker: T):
        self.value = value
        self.marker = marker

    fn drop(mut self):
        println("base")

class Derived[T](Base[T]):
    flag: bool

    fn init(mut self, value: i32, marker: T, flag: bool):
        super.init(value, marker)
        self.flag = flag

fn main():
    item = Derived[bool](42, false, true)
    println(item.value)
