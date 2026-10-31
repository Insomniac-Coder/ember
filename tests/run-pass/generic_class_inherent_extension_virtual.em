#$ test: run-pass
#$ rules: TYP-16, IFC-1, CLS-4, DSP-2
#$ profiles: debug, development, release
#$ stdout: 42
#$ assert-c: contains(em_5Child5valueGda5cc12a188c7619)
#$ assert-c: contains(->slot0)

[T]
open class Base:
    virtual fn value() -> i32:
        return 1

    fn init():
        pass

[T]
class Child(Base[T]):
    fn init():
        super.init()

extend[T] Child[T]:
    override fn value() -> i32:
        return 42

fn main():
    child = Child[bool]()
    base: Base[bool] = child
    println(base.value())
