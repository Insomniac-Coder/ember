#$ test: run-fail
#$ rules: PHIL-8, CELL-5
#$ panics: already
#$ stdout: before
# `[PHIL-8]` — where the property cannot be proved statically but can be enforced safely at run
# time, Ember emits a check: two mutable borrows of one `RefCell`, which the compiler cannot see
# apart, stop at run time instead of being refused.

fn take(c: RefCell[int]) -> int:
    first = c.borrow_mut()
    second = c.borrow_mut()
    return first + second

fn main():
    c = RefCell(1)
    println("before")
    println(take(c))
