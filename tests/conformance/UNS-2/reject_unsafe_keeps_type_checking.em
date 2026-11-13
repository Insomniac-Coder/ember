#$ test: compile-fail
#$ rules: UNS-2
# `[UNS-2]` — `unsafe` permits exactly the operations `[UNS-1]` lists; it does not turn off type
# checking: inside the block a text is no `i32`, and neither is a raw pointer.

fn main():
    xs: Array[int] = [1, 2, 3]
    p = xs.as_mut_span().as_mut_ptr()
    # SAFETY: `xs` holds three elements, and the write is to the first.
    unsafe:
        p.write(9)
        n: i32 = "text"    #$ error[E2020]: expected `i32`, found `StrView`
        m: i32 = p    #$ error[E2020]: expected `i32`, found `*mut i64`
        println(n, m)
