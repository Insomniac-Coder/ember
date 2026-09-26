#$ test: run-pass
#$ rules: EXC-8, EXC-9, EXC-12, EXC-13, EXC-14, TST-15
#$ profiles: debug, release, shipping
#$ stdout: 3

# `[EXC-13](2)` — the receiver is an invariant local. Copies made before the
# loop deliberately prevent `[EXC-3]` static elision while leaving the object's
# identity invariant for the loop-level proof: each `mut self` call writes the
# object, only the counter runs between them, and `self` cannot be re-pointed
# (ODR-072), so one write held for the loop is the same check.

class Counter:
    value: i32

    fn bump(mut self):
        self.value = self.value + 1

fn main():
    owner = Counter(0)
    alias = owner
    receiver = alias
    for i in 0..3:
        receiver.bump()
    println(owner.value)
