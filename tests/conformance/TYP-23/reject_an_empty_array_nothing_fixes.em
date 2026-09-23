#$ test: compile-fail
#$ rules: TYP-23
#$ error[E2060]: cannot infer the element type of `xs`
#$ help: annotate the declaration: `xs: Array[int] = []`
# `[TYP-23]` — a type still open at the end of the function is `E2060`,
# highlighting the first use.

fn main():
    xs = []
    println(len(xs))
