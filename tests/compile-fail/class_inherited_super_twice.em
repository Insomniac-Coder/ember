#$ test: compile-fail
#$ rules: CLS-2, CLS-4
#$ error[E1010]: `super.init(...)` may be called only once

open class Base:
    value: i32

    fn init(value: i32):
        this.value = value

class Derived(Base):
    extra: i32

    fn init(value: i32, extra: i32):
        this.extra = extra
        super.init(value)
        super.init(value)

fn main():
    item = Derived(19, 23)
