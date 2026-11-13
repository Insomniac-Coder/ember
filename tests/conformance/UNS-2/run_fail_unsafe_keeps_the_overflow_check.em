#$ test: run-fail
#$ rules: UNS-2
#$ panics: overflow
# `[UNS-2]` — `unsafe` permits exactly the operations `[UNS-1]` lists; it does not turn off overflow
# checks: the addition inside the block, after the raw write, still panics on overflow.

fn main():
    xs: Array[i32] = [i32.MAX, 1]
    p = xs.as_mut_span().as_mut_ptr()
    # SAFETY: `xs` holds two elements, and the write is to the first.
    unsafe:    #$ warning[L3010]: larger than necessary
        p.write(xs[0])
        total: i32 = xs[0] + xs[1]
        println(total)
