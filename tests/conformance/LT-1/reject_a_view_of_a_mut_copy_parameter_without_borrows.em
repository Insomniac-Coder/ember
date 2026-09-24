#$ test: compile-fail
#$ rules: LT-1, LT-1a
#$ help: write `@borrows(n)` above the declaration
# Not being a source, a `mut` `Copy` parameter can back a returned view only
# when `@borrows` names it.

fn slot(mut n: int) -> ref int:
    return ref n    #$ error[E3062]: the returned view points into `n`, which the result does not borrow

fn main():
    x = 5
    println(slot(x))
