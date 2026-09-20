#$ test: run-pass
#$ rules: TYP-16, CLS-4, DSP-2
#$ profiles: debug, release, shipping
#$ assert-c: contains(static int32_t em_vt_Derived_bool_slot0)
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
