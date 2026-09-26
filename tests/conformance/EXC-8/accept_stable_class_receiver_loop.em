#$ test: run-pass
#$ rules: EXC-8, EXC-9, EXC-11, EXC-12, EXC-13, EXC-14, TST-15
#$ profiles: debug, release, shipping
#$ stdout: 3

# `[EXC-8]` — `counter` has a stable class-handle identity throughout the
# loop. The driver integration test checks the emitted safety-side-table
# classification and protected interval.

class Counter:
    value: i32

    fn bump(mut self):
        self.value = self.value + 1

fn main():
    counter = Counter(0)
    alias = counter
    for i in 0..3:
        counter.bump()
    println(alias.value)
