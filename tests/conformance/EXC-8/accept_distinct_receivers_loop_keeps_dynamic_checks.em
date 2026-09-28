#$ test: run-pass
#$ rules: EXC-9, EXC-12, EXC-13, EXC-14, TST-15
#$ profiles: debug, release, shipping
#$ stdout: 6

# `[EXC-13](5)` — two receiver identities in one loop must not be merged into
# one token. Each `mut self` call remains dynamically checked.

class Counter:
    value: i32

    fn bump(mut self):
        self.value = self.value + 1

    fn print_plus(mut self, extra: i32):
        println(self.value + extra)

# `report` holds a `Counter` while other code (the printing) runs, so a check
# on a `Counter` can fail somewhere and each stays (`[EXC-3]`, ODR-085);
# without it they would all be removed and this loop would show no check.
fn report(mut counter: Counter, extra: i32):
    counter.print_plus(extra)

fn main():
    left = Counter(0)
    right = Counter(0)
    left_alias = left
    right_alias = right
    for i in 0..3:
        left.bump()
        right.bump()
    report(left_alias, right_alias.value)
