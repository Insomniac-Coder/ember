#$ test: run-pass
#$ rules: EXC-5, EXC-15, CLS-7
#$ profiles: debug, development, release
#$ stdout: 3 [1, 2, 3]
# `[EXC-5]` — a `mut self` method calling another `mut self` method on `self`
# reborrows the write access its caller holds and checks nothing again. It
# panicked before (D-359): the callee took the whole object again at entry.

class Counter:
    n: int
    log: Array[int]

    fn init():
        this.n = 0
        this.log = []

    fn bump():
        this.n += 1
        this.log.push(owns this.n)

    fn twice():
        this.bump()
        this.bump()

fn touch(ref mut c: Counter):
    c.bump()

fn main():
    c = Counter()
    c.twice()
    touch(ref mut c)
    println(c.n, c.log)
