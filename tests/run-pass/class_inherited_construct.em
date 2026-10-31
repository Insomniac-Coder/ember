#$ test: run-pass
#$ rules: CLS-1, CLS-2, CLS-4
#$ profiles: debug, development, release
#$ assert-c: contains(ember_obj_new(&em_ti_7Derived))
#$ assert-c: contains(em_7Derived4init)
#$ assert-c: contains(em_4Base4init)
#$ stdout: 42

open class Base:
    value: i32

    fn init(value: i32):
        this.value = value

class Derived(Base):
    extra: i32

    fn init(value: i32, extra: i32):
        this.extra = extra
        super.init(value)

fn main():
    item = Derived(19, 23)
    println(item.value + item.extra)
