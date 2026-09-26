#$ test: run-pass
#$ rules: EXC-5, EXC-15, CLS-7
#$ profiles: debug, release, shipping
#$ stdout: 3 [1, 2, 3]
# `[EXC-5]` — a `mut self` method calling another `mut self` method on `self`
# reborrows the write access its caller holds and checks nothing again. It
# panicked before (D-359): the callee took the whole object again at entry.

class Counter:
    n: int
    log: Array[int]

    fn init(mut self):
        self.n = 0
        self.log = []

    fn bump(mut self):
        self.n += 1
        self.log.push(self.n)

    fn twice(mut self):
        self.bump()
        self.bump()

fn touch(mut c: Counter):
    c.bump()

fn main():
    c = Counter()
    c.twice()
    touch(c)
    println(c.n, c.log)
