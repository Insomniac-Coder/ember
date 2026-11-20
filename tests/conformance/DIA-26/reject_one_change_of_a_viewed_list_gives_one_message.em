#$ test: compile-fail
#$ rules: DIA-26, BRW-1
# `[DIA-26]` — one mistake gives one message: `xs[0] = 9` while `m`, a changeable view of `xs`, is still
# used reads `xs`'s length for the index check and writes its element, and is one `E3021`, not two.

fn main():
    xs = [1, 2, 3]
    m = xs.as_mut_span()
    xs[0] = 9    #$ error[E3021]
    println(m[0])
