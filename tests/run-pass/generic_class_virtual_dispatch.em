#$ test: run-pass
#$ rules: TYP-16, CLS-4, DSP-2
#$ profiles: debug, development, release
#$ assert-c: contains(static int32_t em_vt_7DerivedGda5cc12a188c7619_slot0)
#$ assert-c: contains(->slot0)
#$ stdout: 42

[T]
open class Base:
    virtual fn value() -> i32:
        return 1

    fn init():
        pass

[T]
class Derived(Base[T]):
    fn init():
        super.init()

    override fn value() -> i32:
        return 42

fn main():
    derived = Derived[bool]()
    base: Base[bool] = derived
    println(base.value())
