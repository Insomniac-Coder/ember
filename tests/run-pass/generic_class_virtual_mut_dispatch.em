#$ test: run-pass
#$ rules: TYP-16, CLS-4, DSP-2
#$ profiles: debug, development, release
#$ assert-c: contains(static int32_t em_vt_7DerivedGda5cc12a188c7619_slot0)
#$ assert-c: contains(->slot0)
#$ stdout: 42

[T]
open class Base:
    value: i32

    virtual fn bump() -> i32:
        this.value = this.value + 1
        return this.value

    fn init():
        this.value = 1

[T]
class Derived(Base[T]):
    fn init():
        super.init()

    override fn bump() -> i32:
        this.value = this.value + 41
        return this.value

fn main():
    derived = Derived[bool]()
    base: Base[bool] = derived
    println(base.bump())
