#$ test: run-pass
#$ rules: TYP-16, CLS-2, CLS-4, CLS-6, DRP-1
#$ profiles: debug, development, release
#$ stdout: 42
#$ stdout: base

open class Base:
    value: i32

    fn init(value: i32):
        this.value = value

    fn drop():
        println("base")

[T]
class Derived(Base):
    marker: T

    fn init(value: i32, marker: T):
        this.marker = marker
        super.init(value)

fn main():
    item = Derived[bool](42, false)
    println(item.value)
