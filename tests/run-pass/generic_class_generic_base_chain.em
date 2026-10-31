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
open class Middle(Base[T]):
    fn init(value: T):
        super.init(value)

[T]
class Leaf(Middle[T]):
    fn init(value: T):
        super.init(value)

fn main():
    item = Leaf[i32](42)
    println(item.value)
