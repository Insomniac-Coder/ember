#$ test: run-fail
#$ rules: EXC-5, EXC-15, EXC-1
#$ profiles: debug, development, release
#$ panics: exclusivity violation: write access to Counter.log while a write access to Counter.log is active
# `[EXC-5]` — only a call on `self` itself is a reborrow. The same object reached
# through another handle is a second write access to every field, and panics.

class Counter:
    log: Array[int]

    fn init():
        this.log = []

    fn bump():
        this.log.push(1)

    fn bump_other(other: Counter):
        this.log.push(0)
        alias = other
        alias.bump()

fn main():
    c = Counter()
    c.bump_other(c)
