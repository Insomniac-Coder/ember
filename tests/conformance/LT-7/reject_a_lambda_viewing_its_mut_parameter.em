#$ test: compile-fail
#$ rules: LT-7, LT-1a
#$ help: a lambda cannot say `from`
# A lambda's `mut int` is no source, so its result cannot view it; the help
# does not offer `@borrows`, which a lambda cannot carry.

fn main():
    g: fn(ref mut int) -> ref int = fn(ref mut x) => ref x    #$ error[E3062]: the returned view points into `x`, which the result does not point into
    y = 5
    r = g(y)
    y = 9
    println(r)
