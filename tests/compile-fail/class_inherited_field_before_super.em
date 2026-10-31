#$ test: compile-fail
#$ rules: CLS-2, CLS-4, CLS-11
#$ error[E2102]: inherited class field is read before `super.init(...)`

open class Base:
    value: i32

    fn init(value: i32):
        this.value = value

class Derived(Base):
    extra: i32

    fn init(extra: i32):
        this.extra = extra
        println(this.value)
        super.init(19)

fn main():
    item = Derived(23)
