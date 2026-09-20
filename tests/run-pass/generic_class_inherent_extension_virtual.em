#$ test: run-pass
#$ rules: TYP-16, IFC-1, CLS-4, DSP-2
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: contains(em_Child_bool_value)
#$ assert-c: contains(->slot0)

open class Base[T]:
    virtual fn value(self) -> i32:
        return 1

    fn init(mut self):
        pass

class Child[T](Base[T]):
    fn init(mut self):
        super.init()

extend[T] Child[T]:
    override fn value(self) -> i32:
        return 42

fn main():
    child = Child[bool]()
    base: Base[bool] = child
    println(base.value())
