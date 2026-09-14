#$ test: run-pass
#$ rules: CLS-1, CLS-2, CLS-4
#$ profiles: debug, release, shipping
#$ assert-c: contains(ember_obj_new(&em_ti_Derived))
#$ assert-c: contains(em_Derived_init)
#$ assert-c: contains(em_Base_init)
#$ stdout: 42

open class Base:
    value: i32

    fn init(mut self, value: i32):
        self.value = value

class Derived(Base):
    extra: i32

    fn init(mut self, value: i32, extra: i32):
        super.init(value)
        self.extra = extra

fn main():
    item = Derived(19, 23)
    println(item.value + item.extra)
