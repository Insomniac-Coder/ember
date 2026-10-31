#$ test: run-pass
#$ rules: TYP-16, CLS-2, CLS-4, CLS-6, DRP-1
#$ profiles: debug, development, release
#$ stdout: 42
#$ stdout: base

[T]
open class Base:
    value: i32
    marker: T

    fn init(value: i32, marker: T):
        this.value = value
        this.marker = marker

    fn drop():
        println("base")

[T]
class Derived(Base[T]):
    flag: bool

    fn init(value: i32, marker: T, flag: bool):
        this.flag = flag
        super.init(value, marker)

fn main():
    item = Derived[bool](42, false, true)
    println(item.value)
