#$ test: run-pass
#$ rules: TYP-16, CLS-4, DSP-2
#$ profiles: debug, release, shipping
#$ assert-c: contains(static int32_t em_vt_Derived_bool_slot0)
#$ assert-c: contains(->slot0)
#$ stdout: 42

open class Base[T]:
    value: i32

    virtual fn bump(mut self) -> i32:
        self.value = self.value + 1
        return self.value

    fn init(mut self):
        self.value = 1

class Derived[T](Base[T]):
    fn init(mut self):
        super.init()

    override fn bump(mut self) -> i32:
        self.value = self.value + 41
        return self.value

fn main():
    derived = Derived[bool]()
    base: Base[bool] = derived
    println(base.bump())
