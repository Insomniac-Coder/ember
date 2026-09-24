#$ test: compile-fail
#$ rules: LT-7, LT-1
# The source argument stays borrowed while the result of a call through a
# callable value lives; the `int` argument does not.

fn head(xs: Array[int], n: int) -> Span[int]:
    return xs[..n]

fn main():
    h: fn(Array[int], int) -> Span[int] = head
    xs = [1, 2, 3]
    n = 2
    v = h(xs, n)
    n = 3
    xs.push(9)    #$ error[E3021]: `xs` is borrowed here and mutably borrowed elsewhere
    println(v, n)
