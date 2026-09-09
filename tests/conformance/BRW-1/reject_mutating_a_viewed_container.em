#$ test: compile-fail
#$ rules: BRW-1, SPN-1, UNS-4
## "while shared borrows are live the owner may read (and copy) but not write,
## move, or drop." A `Span[T]` points **into** its container, so the container
## is borrowed for as long as the view is held — and `push` may reallocate,
## which would leave the view pointing at freed memory. `[PHIL-10]` says Safe
## Ember cannot do that.

fn main():
    a: Array[i32] = Array[i32]()
    a.push(1)
    v: Span[i32] = a
    a.push(2)              #$ error[E3021]: `a` is borrowed here and mutably borrowed elsewhere
    println(v[0])
