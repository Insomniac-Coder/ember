#$ test: run-pass
#$ rules: EXC-9, EXC-12, EXC-13, EXC-14, TST-15
#$ profiles: debug, release, shipping
#$ stdout: 3

# `[EXC-13](4)` — dispatch through a virtual method has no local effect proof
# at the loop pass, so it must remain a per-access check.

open class Counter:
    value: i32

    virtual fn bump(mut self):
        self.value = self.value + 1

    fn init(mut self):
        self.value = 0

class Derived(Counter):
    override fn bump(mut self):
        self.value = self.value + 1

    fn init(mut self):
        super.init()

fn main():
    counter: Counter = Derived()
    alias = counter
    for i in 0..3:
        counter.bump()
    println(alias.value)
