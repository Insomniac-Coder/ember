#$ test: run-fail
#$ rules: EXC-19, EXC-15, EXC-1
#$ profiles: debug, development, release
#$ panics: exclusivity violation: write access to Counter.items while a read access to Counter.items is active
# `[EXC-19]` — a method is quiet when everything it calls is quiet too, like
# `bump` calling `step`. Its `mut self` call is only checked, and the check
# still sees the loop reading the same object's field through another handle.

class Counter:
    items: Array[int]
    count: int

    fn init():
        this.items = [1, 2, 3]
        this.count = 0

    fn bump():
        this.count = step(this.count)

fn step(x: int) -> int:
    return x + 1

fn main():
    c = Counter()
    alias = c
    for x in c.items:
        alias.bump()
        println(x)
