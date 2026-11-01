#$ test: run-fail
#$ rules: EXC-1, EXC-9, EXC-10
#$ profiles: debug, development, release
#$ panics: exclusivity violation: write access to Counter.log while a write access to Counter.log is active

# `[EXC-10]`, `[EXC-9]` — `run` changes its object (`this.log.push(0)`), so its call holds it
# for writing, and `other` is the same object: the nest's one check fails at the inner call, where the first
# per-call check failed before.

class Counter:
    value: i32
    log: Array[int]

    fn bump():
        this.value = this.value + 1

    fn run(ref mut other: Counter):
        this.log.push(owns 0)
        for r in 0..2:
            for i in 0..2:
                other.bump()

fn main():
    counter = Counter(0, [])
    alias = counter
    counter.run(ref mut alias)
