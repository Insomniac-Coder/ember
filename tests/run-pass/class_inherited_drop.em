#$ test: run-pass
#$ rules: CLS-4, CLS-6, DRP-1, OWN-2
#$ profiles: debug, development, release
#$ assert-c-order: "em_7Derived4drop(&handle)" then "em_4Base4drop(&base_handle_0)"
#$ stdout: derived
#$ base

open class Base:
    value: i32

    fn init(value: i32):
        this.value = value

    fn drop():
        println("base")

class Derived(Base):
    extra: i32

    fn init(value: i32, extra: i32):
        this.extra = extra
        super.init(value)

    fn drop():
        println("derived")

fn main():
    item = Derived(19, 23)
