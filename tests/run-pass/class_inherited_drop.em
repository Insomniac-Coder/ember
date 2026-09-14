#$ test: run-pass
#$ rules: CLS-4, CLS-6, DRP-1, OWN-2
#$ profiles: debug, release, shipping
#$ assert-c-order: "em_Derived_drop(&handle)" then "em_Base_drop(&base_handle_0)"
#$ stdout: derived
#$ base

open class Base:
    value: i32

    fn init(mut self, value: i32):
        self.value = value

    fn drop(mut self):
        println("base")

class Derived(Base):
    extra: i32

    fn init(mut self, value: i32, extra: i32):
        super.init(value)
        self.extra = extra

    fn drop(mut self):
        println("derived")

fn main():
    item = Derived(19, 23)
