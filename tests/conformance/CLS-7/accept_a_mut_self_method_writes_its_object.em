#$ test: run-pass
#$ rules: CLS-7, EXC-15, FN-9
#$ stdout: 0 [1] 7
# `[CLS-7]` — inside a class method `self` is a handle to the object; `mut self`
# writes its fields. Re-pointing a caller's handle is a `mut` parameter's
# (`[FN-9]`), not `self`'s (ODR-072).

class Counter:
    value: int
    log: Array[int]

    fn init(value: int):
        this.value = value
        this.log = []

    fn reset():
        this.value = 0
        this.log.push(owns 1)

fn replace(ref mut c: Counter):
    c = Counter(7)

fn main():
    c = Counter(5)
    c.reset()
    kept = c
    replace(ref mut c)
    println(kept.value, kept.log, c.value)
