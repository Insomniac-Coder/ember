#$ test: compile-fail
#$ rules: DIA-21
#$ error[E0100]: a slice is written `xs[a..b]` in Ember
#$ help: write `..`
# `[DIA-21]` — Python's `xs[a:b]` is answered with the Ember form.

fn main():
    xs = [1, 2, 3]
    println(xs[1:3])
