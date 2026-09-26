#$ test: run-pass
#$ rules: EXC-9, EXC-12, EXC-13, EXC-14, TST-15
#$ profiles: debug, release, shipping
#$ stdout: 6

# `[EXC-13](3)` — publishing another copy of the receiver in the loop defeats
# the stable-receiver proof. The source remains safe because the ordinary
# per-access exclusivity check stays in place.

class Counter:
    value: i32

    fn bump(mut self):
        self.value = self.value + 1

fn main():
    counter = Counter(0)
    alias = counter
    published = counter
    for i in 0..3:
        published = counter
        counter.bump()
    println(alias.value + published.value)
