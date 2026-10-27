#$ test: compile-fail
#$ rules: R17, R14
#$ help: [T: Add[T, Output = T]]
## R17: a `pub` function's bounds are written, because they are its promise to other packages; the
## compiler never decides them from a body (AN-69, E2040, today's code).

[T]
pub fn total(a: T, b: T) -> T:
    return a + b    #$ error[E2040]: `total` is `pub`, so its bound is written: write `[T: Add[T, Output = T]]` in place of `[T]` above `pub fn total(a: T, b: T) -> T:`

fn main():
    println(total(2, 3))
