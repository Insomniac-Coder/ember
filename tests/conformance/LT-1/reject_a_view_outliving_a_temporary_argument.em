#$ test: compile-fail
#$ rules: LT-1, EXP-4
#$ help: bind the value to a variable first
# A temporary argument lives to the end of its statement; a result borrowing
# it cannot outlive that, and the error names the expression, not a temporary.

fn head(xs: Array[int]) -> Span[int]:
    return xs[..2]

fn make() -> Array[int]:
    return [5, 6, 7]

fn main():
    t = head(make())    #$ error[E3060]: this temporary is dropped at the end of its statement
    println(t[0])
