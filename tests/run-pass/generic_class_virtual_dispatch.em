#$ test: run-pass
#$ rules: TYP-16, CLS-4, DSP-2
#$ profiles: debug, development, release
#$ assert-c: contains(static int32_t em_vt_7DerivedGda5cc12a188c7619_slot0)
#$ assert-c: contains(->slot0)
#$ stdout: 42

open class Base[T]:
    virtual fn value(self) -> i32:
        return 1

    fn init(mut self):
        pass

class Derived[T](Base[T]):
    fn init(mut self):
        super.init()

    override fn value(self) -> i32:
        return 42

fn main():
    derived = Derived[bool]()
    base: Base[bool] = derived
    println(base.value())
