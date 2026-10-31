#$ test: compile-fail
#$ rules: CLS-2, CLS-4
#$ error[E2100]: derived class `init` must call `super.init(...)` exactly once before returning

open class Base:
    value: i32

    fn init(value: i32):
        this.value = value

class Derived(Base):
    extra: i32

    fn init(extra: i32):
        this.extra = extra

fn main():
    item = Derived(23)
