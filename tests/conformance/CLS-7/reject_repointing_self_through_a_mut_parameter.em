#$ test: compile-fail
#$ rules: CLS-7, EXC-15, FN-9
#$ error[E2103]: cannot re-point `this` in a class method
# `[CLS-7]` (ODR-072) — `self` names the object a method was called on for the
# whole call, however the handle is reached: passing it to a `mut` handle
# parameter could re-point it as surely as `self = …` (F5).

class Bag:
    items: Array[int]

    fn init():
        this.items = [1, 2, 3]

    fn clobber(other: Bag):
        repoint(this, other)
        this.items = [9]

fn repoint(ref mut c: Bag, to: Bag):
    c = to

fn main():
    a = Bag()
    a.clobber(Bag())
