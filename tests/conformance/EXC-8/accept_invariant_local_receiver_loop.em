#$ test: run-pass
#$ rules: EXC-8, EXC-9, EXC-12, EXC-13, EXC-14, TST-15
#$ profiles: debug, release, shipping
#$ stdout: 3

# `[EXC-13](2)` — the receiver is loaded from an invariant local. Copies made
# before the loop deliberately prevent `[EXC-3]` static elision while leaving
# the class-object identity invariant for the loop-level proof.

class Counter:
    value: i32

    fn bump(mut self):
        self.value = self.value + 1

class Holder:
    child: Counter

fn main():
    owner = Holder(Counter(0))
    alias = owner
    receiver = alias
    for i in 0..3:
        receiver.child.bump()
    println(owner.child.value)
