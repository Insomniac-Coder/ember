#$ test: compile-fail
#$ rules: LT-1, BRW-1
# The caller keeps the argument borrowed while the result borrows it.

fn head(xs: Array[int]) -> Span[int]:
    return xs[..2]

fn main():
    xs = [1, 2, 3]
    s = head(xs)
    xs.push(4)    #$ error[E3021]: `xs` is borrowed here and mutably borrowed elsewhere
    println(s[0])
