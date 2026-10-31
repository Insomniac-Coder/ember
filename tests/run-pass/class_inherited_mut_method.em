#$ test: run-pass
#$ rules: CLS-4, CLS-7, EXC-1
#$ profiles: debug, development, release
#$ assert-c: contains(em_4Base4bump)
#$ assert-c: !contains(ember_retain)
#$ stdout: 7

open class Base:
    value: i32

    fn init(value: i32):
        this.value = value

    fn bump(amount: i32):
        this.value = this.value + amount

class Derived(Base):
    extra: i32

    fn init(value: i32, extra: i32):
        this.extra = extra
        super.init(value)

fn main():
    item = Derived(2, 5)
    item.bump(5)
    println(item.value)
