#$ test: run-pass
#$ rules: TYP-3
#$ profiles: debug, development
#$ stdout: A
# `[TYP-3]` — inside `unsafe` four bytes may be seen as a `char`, and the programmer answers for
# their being a Unicode scalar value: 65 is.

fn main():
    values: Array[u32] = Array[u32]()
    values.push(owns 65)
    m = values.as_mut_span()
    p: *mut u32 = m.as_mut_ptr()
    unsafe:  # SAFETY: 65 is a Unicode scalar value, the only kind of bit pattern a `char` may hold.
        q = p as *mut char
        println(read(q, 0))
