#$ test: run-pass
#$ rules: CLS-7, EXC-15, FN-9
#$ stdout: 0 [1] 7
# `[CLS-7]` — inside a class method `self` is a handle to the object; `mut self`
# writes its fields. Re-pointing a caller's handle is a `mut` parameter's
# (`[FN-9]`), not `self`'s (ODR-072).

class Counter:
    value: int
    log: Array[int]

    fn init(mut self, value: int):
        self.value = value
        self.log = []

    fn reset(mut self):
        self.value = 0
        self.log.push(1)

fn replace(mut c: Counter):
    c = Counter(7)

fn main():
    c = Counter(5)
    c.reset()
    kept = c
    replace(c)
    println(kept.value, kept.log, c.value)
