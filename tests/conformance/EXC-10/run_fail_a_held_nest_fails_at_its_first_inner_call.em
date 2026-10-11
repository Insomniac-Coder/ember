#$ test: run-fail
#$ rules: EXC-1, EXC-9, EXC-10
#$ profiles: debug, release, shipping
#$ panics: exclusivity violation: write access to Counter.log while a write access to Counter.log is active

# `[EXC-10]`, `[EXC-9]` — `run` holds `self` for writing, and `other` is the same
# object: the nest's one check fails at the inner call, where the first
# per-call check failed before.

class Counter:
    value: i32
    log: Array[int]

    fn bump(mut self):
        self.value = self.value + 1

    fn run(mut self, mut other: Counter):
        for r in 0..2:
            for i in 0..2:
                other.bump()

fn main():
    counter = Counter(0, [])
    alias = counter
    counter.run(alias)
