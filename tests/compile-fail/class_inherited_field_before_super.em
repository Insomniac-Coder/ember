#$ test: compile-fail
#$ rules: CLS-2, CLS-4, CLS-11
#$ error[E2102]: inherited class field is read before `super.init(...)`

open class Base:
    value: i32

    fn init(mut self, value: i32):
        self.value = value

class Derived(Base):
    extra: i32

    fn init(mut self, extra: i32):
        self.extra = extra
        println(self.value)
        super.init(19)

fn main():
    item = Derived(23)
