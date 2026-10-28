#$ test: compile-fail
#$ rules: BRW-14, BRW-15
#$ help: write `d = ref mut xs.as_mut_span()`
## design rules: R2, R5
## R2, R5: a plain `d = xs.as_mut_span()` only reads the span's elements; a write through it is
## refused (E3023) with the line to write (verdict 34; `V4/rules.md` section 3's message in this
## program's names).

fn main():
    xs = [1, 2]
    d = xs.as_mut_span()
    d[0] = 0                   #$ error[E3023]: `d[0] = 0` writes an element of `xs`, but `d` only reads it
    println(xs[0])
