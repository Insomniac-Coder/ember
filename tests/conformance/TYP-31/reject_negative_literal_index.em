#$ test: compile-fail
#$ rules: TYP-31, LEX-24
#$ profiles: debug
#$ error[E2011]: negative literal index
#$ help: count from the length: `xs[xs.len() - 1]`
# Python's `xs[-1]` does not carry over: the fix-its are `xs.last()` and
# `xs[xs.len() - 1]`.

fn main():
    xs = [10, 20, 30]
    println(xs[-1])
