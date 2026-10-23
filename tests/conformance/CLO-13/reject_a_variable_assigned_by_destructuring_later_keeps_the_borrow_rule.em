#$ test: compile-fail
#$ rules: CLO-13
# `[CLO-13]` — a variable assigned after the lambda is made keeps the borrow rule, however it is
# assigned: a destructuring assignment too. The scan of the body's assignments saw only a single
# name on the left, so `x` was copied into the stored lambda and `f()` gave 1 after `x` became 2,
# a difference a copy is never to show.

fn main():
    x = 1
    y = 0
    f: fn() -> int = fn() => x  #$ error[E3026]: this lambda borrows `x`, which is assigned later
    (x, y) = (2, 3)
    println(f(), x, y)
