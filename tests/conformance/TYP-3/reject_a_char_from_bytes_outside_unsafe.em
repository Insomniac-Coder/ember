#$ test: compile-fail
#$ rules: TYP-3
#$ profiles: debug
#$ error[E3100]: requires `unsafe`
#$ error[E3100]: `read` needs an `unsafe` block
# `[TYP-3]` — seeing four bytes as a `char` goes through a raw pointer, whose cast and read are both
# `unsafe`: the programmer there answers for the bytes being a Unicode scalar value.

fn main():
    values: Array[u32] = Array[u32]()
    values.push(owns 55296)
    m = values.as_mut_span()
    p: *mut u32 = m.as_mut_ptr()
    q = p as *mut char
    println(read(q, 0))
