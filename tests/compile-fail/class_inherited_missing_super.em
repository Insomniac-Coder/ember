#$ test: compile-fail
#$ rules: CLS-2, CLS-4
#$ error[E2100]: derived class `init` must call `super.init(...)` exactly once before returning

open class Base:
    value: i32

    fn init(mut self, value: i32):
        self.value = value

class Derived(Base):
    extra: i32

    fn init(mut self, extra: i32):
        self.extra = extra

fn main():
    item = Derived(23)
