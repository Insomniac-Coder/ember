#$ test: compile-fail
#$ rules: LT-7, LT-1a
#$ help: a lambda cannot carry `@borrows`
# A lambda's `mut int` is no source, so its result cannot view it; the help
# does not offer `@borrows`, which a lambda cannot carry.

fn main():
    g: fn(mut int) -> ref int = fn(mut x) => ref x    #$ error[E3062]: the returned view points into `x`, which the result does not borrow
    y = 5
    r = g(y)
    y = 9
    println(r)
