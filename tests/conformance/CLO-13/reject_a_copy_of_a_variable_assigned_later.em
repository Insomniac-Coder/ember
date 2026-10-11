#$ test: compile-fail
#$ rules: CLO-13, CLO-3

# `[CLO-13]` — a captured variable assigned after the lambda keeps the borrow
# rule, which an owned callable value cannot hold; the help is `owned fn`.

fn main():
    k = 7
    g: fn() -> int = fn() => k    #$ error[E3063]: this lambda borrows `k`, which is assigned later, so it may not be stored as a value
    k = 8
    println(g(), k)
