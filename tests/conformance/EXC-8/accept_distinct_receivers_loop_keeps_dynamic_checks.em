#$ test: run-pass
#$ rules: EXC-9, EXC-12, EXC-13, EXC-14, TST-15
#$ profiles: debug, release, shipping
#$ stdout: 6

# `[EXC-13](5)` — two receiver identities in one loop must not be merged into
# one token. Each member call remains dynamically checked.

class Counter:
    value: i32

    fn bump(mut self):
        self.value = self.value + 1

class Holder:
    child: Counter

fn main():
    left = Holder(Counter(0))
    right = Holder(Counter(0))
    left_alias = left
    right_alias = right
    for i in 0..3:
        left.child.bump()
        right.child.bump()
    println(left_alias.child.value + right_alias.child.value)
