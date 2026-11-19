#$ test: compile-fail
#$ rules: LT-1, BRW-1
# The caller keeps the argument borrowed while the result borrows it.

fn head(xs: Array[int]) -> Span[int]:
    return xs[..2]

fn main():
    xs = [1, 2, 3]
    s = head(xs)
    xs.push(owns 4)
    println(s[0])    #$ error[E3060]
