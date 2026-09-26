#$ test: run-fail
#$ rules: EXC-5, EXC-15, EXC-1
#$ profiles: debug, release, shipping
#$ panics: exclusivity violation: write access to Counter.log while a write access to Counter.log is active
# `[EXC-5]` — only a call on `self` itself is a reborrow. The same object reached
# through another handle is a second write access to every field, and panics.

class Counter:
    log: Array[int]

    fn init(mut self):
        self.log = []

    fn bump(mut self):
        self.log.push(1)

    fn bump_other(mut self, other: Counter):
        alias = other
        alias.bump()

fn main():
    c = Counter()
    c.bump_other(c)
