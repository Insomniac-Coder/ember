#$ test: run-pass
#$ rules: TYP-16, CLS-2, CLS-4
#$ profiles: debug, release, shipping
#$ stdout: 42

open class Base[T]:
    value: T

    fn init(mut self, value: T):
        self.value = value

open class Middle[T](Base[T]):
    fn init(mut self, value: T):
        super.init(value)

class Leaf[T](Middle[T]):
    fn init(mut self, value: T):
        super.init(value)

fn main():
    item = Leaf[i32](42)
    println(item.value)
