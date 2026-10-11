#$ test: run-pass
#$ rules: TYP-16, CLS-2, CLS-4
#$ profiles: debug, release, shipping
#$ stdout: 42

open class Base:
    value: i32

    fn init(mut self, value: i32):
        self.value = value

class Derived[T](Base):
    marker: T

    fn init(mut self, value: i32, marker: T):
        self.marker = marker
        super.init(value)

fn main():
    item = Derived[bool](42, false)
    println(item.value)
