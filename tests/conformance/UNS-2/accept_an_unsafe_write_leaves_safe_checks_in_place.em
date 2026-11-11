#$ test: run-pass
#$ rules: UNS-2
#$ stdout: 9 2
# `[UNS-2]` — `unsafe` permits the raw write and nothing else.

fn main():
    xs: Array[int] = [1, 2, 3]
    p = xs.as_mut_span().as_mut_ptr()
    # SAFETY: `xs` holds three elements, and the write is to the first.
    unsafe:
        p.write(9)
    println(xs[0], xs[1])
