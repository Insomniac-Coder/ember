#$ test: run-pass
#$ rules: CELL-7
# `[CELL-7]` — early return through a live guard releases it. `foo` borrows
# through a `mut` receiver (a `ref mut` to the caller's cell, so the counter
# updated is the caller's), returns early with the guard live, and the guard's
# drop runs on the return path. The caller then borrows cleanly.

fn foo(mut c: RefCell[i32]) -> i32:
    with _g = c.borrow():
        return 7
    return 8

fn main():
    c: RefCell[i32] = RefCell(1)
    println(foo(c))
    with _h = c.borrow_mut():
        println(2)
#$ stdout: 7
#$ 2
