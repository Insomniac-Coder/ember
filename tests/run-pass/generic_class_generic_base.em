#$ test: run-pass
#$ rules: TYP-16, CLS-2, CLS-4
#$ profiles: debug, development, release
#$ stdout: 42

[T]
open class Base:
    value: T

    fn init(value: T):
        this.value = value

[T]
class Derived(Base[T]):
    marker: bool

    fn init(value: T, marker: bool):
        this.marker = marker
        super.init(value)

fn main():
    item = Derived[i32](42, false)
    println(item.value)
