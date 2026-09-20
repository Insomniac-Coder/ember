#$ test: run-pass
#$ rules: TYP-16, CLS-2, CLS-4, CLS-6, DRP-1
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ stdout: base

open class Base:
    value: i32

    fn init(mut self, value: i32):
        self.value = value

    fn drop(mut self):
        println("base")

class Derived[T](Base):
    marker: T

    fn init(mut self, value: i32, marker: T):
        super.init(value)
        self.marker = marker

fn main():
    item = Derived[bool](42, false)
    println(item.value)
