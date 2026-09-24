#$ test: compile-fail
#$ rules: LT-1, LT-1a
#$ help: return the value instead of a view of it
# ODR-024 — `x: T` counts as `Copy` for elision, so no instantiation lets a
# returned view point into it, and `@borrows` cannot name it either: even at
# `T = String`, where `x` is passed by address, the result may not borrow it.

fn keep[T](x: T) -> ref T:
    return ref x    #$ error[E3062]: the returned view points into `x`, which the result does not borrow

fn main():
    s: String = "zz"
    r = keep(s)
    println(r)
