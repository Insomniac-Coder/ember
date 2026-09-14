#$ test: compile-fail
#$ rules: CLS-2, CLS-4
#$ error[E1010]: `super.init(...)` may be called only once

open class Base:
    value: i32

    fn init(mut self, value: i32):
        self.value = value

class Derived(Base):
    extra: i32

    fn init(mut self, value: i32, extra: i32):
        super.init(value)
        super.init(value)
        self.extra = extra

fn main():
    item = Derived(19, 23)
