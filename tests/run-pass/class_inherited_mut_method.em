#$ test: run-pass
#$ rules: CLS-4, CLS-7, EXC-1
#$ profiles: debug, release, shipping
#$ assert-c: contains(em_Base_bump)
#$ assert-c: !contains(ember_retain)
#$ stdout: 7

open class Base:
    value: i32

    fn init(mut self, value: i32):
        self.value = value

    fn bump(mut self, amount: i32):
        self.value = self.value + amount

class Derived(Base):
    extra: i32

    fn init(mut self, value: i32, extra: i32):
        super.init(value)
        self.extra = extra

fn main():
    item = Derived(2, 5)
    item.bump(5)
    println(item.value)
