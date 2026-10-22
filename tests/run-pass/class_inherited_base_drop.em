#$ test: run-pass
#$ rules: CLS-4, CLS-6, DRP-1, OWN-2
#$ profiles: debug, development, release
#$ assert-c: contains(em_drop_adapter_7Derived)
#$ assert-c: contains(em_4Base4drop(&)
#$ stdout: base

open class Base:
    value: i32

    fn init(mut self, value: i32):
        self.value = value

    fn drop(mut self):
        println("base")

class Derived(Base):
    extra: i32

    fn init(mut self, value: i32, extra: i32):
        self.extra = extra
        super.init(value)

fn main():
    item = Derived(19, 23)
