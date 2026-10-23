#$ test: compile-fail
#$ rules: IDE-3, CLO-6
# `[IDE-3]` — every stage after parsing gives a whole result for a file with errors: a type error in
# one function stops nothing else, so the move check and the lints run on every function no error
# is in, and report what they find there. A function with an error, and the instance of a generic
# made only for its wrong argument, are not checked further: nothing is reported twice.

fn consume(owned values: Array[i32]) -> i32:
    return values[0]

fn twice(f: fn() -> i32) -> i32:
    return f() + f()

fn moves_twice():
    xs: Array[int] = [1]
    ys = xs
    println(xs.len(), ys.len())  #$ error[E3040]: `xs` has been moved out of

fn reads_nothing():
    k = 3  #$ warning[L1001]: `k` is never read

fn wrong_type() -> int:
    return "three"  #$ error[E2020]: expected `i64`, found `str`

fn main():
    values: Array[i32] = Array[i32]()
    values.push(7)
    task = owned fn() => consume(values)
    println(twice(task))  #$ error[E3030]: closure would move a captured value out
    moves_twice()
    reads_nothing()
    println(wrong_type())
