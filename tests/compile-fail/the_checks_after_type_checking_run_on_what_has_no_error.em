#$ test: compile-fail
#$ rules: IDE-3, CLO-6
# `[IDE-3]` — every stage after parsing gives a whole result for a file with errors: a type error in
# one function stops nothing else, so the move check and the lints run on every function no error
# is in, and report what they find there. A function with an error, and the instance of a generic
# made only for its wrong argument, are not checked further: nothing is reported twice.

fn consume(owns values: Array[i32]) -> i32:
    return values[0]

fn twice(f: fn() -> i32) -> i32:
    return f() + f()

fn moves_twice():
    xs: Array[int] = [1]
    ys = xs
    println(xs.len(), ys.len())  #$ error[E3040]: `xs` was handed over to `ys` on line 16 and is used on line 17; to keep both, write `ys = copy of xs` (a copy of your own) or `ys = shares xs` (two owners of one value)

fn reads_nothing():
    k = 3  #$ warning[L1001]: `k` is never read

fn wrong_type() -> int:
    return "three"  #$ error[E2020]: expected `i64`, found `StrView`

fn main():
    values: Array[i32] = Array[i32]()
    values.push(owns 7)
    task = owned fn() => consume(owns values)
    println(twice(task))  #$ error[E3013]: this function gives `values` away, so it can run only once, and `twice` may run it more than once
    moves_twice()
    reads_nothing()
    println(wrong_type())
