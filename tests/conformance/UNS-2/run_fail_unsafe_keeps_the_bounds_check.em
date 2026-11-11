#$ test: run-fail
#$ rules: UNS-2, UNS-1
#$ panics: index 3 is out of bounds for a length of 3
# `[UNS-2]` — `unsafe` permits exactly the operations it lists; it does not turn off bounds checks
# on safe types.

fn main():
    xs: Array[int] = [1, 2, 3]
    p = xs.as_mut_span().as_mut_ptr()
    i = xs.len()
    # SAFETY: `xs` holds three elements, and the write is to the first.
    unsafe:
        p.write(9)
    println(xs[i])
