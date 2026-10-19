#$ test: run-pass
#$ rules: TYP-2
#$ profiles: debug, development
#$ stdout: true false
# `[TYP-2]` — inside `unsafe` a byte may be seen as a `bool`, and the programmer answers for its
# being 0 or 1: these two are.

fn main():
    values: Array[u8] = Array[u8]()
    values.push(1)
    values.push(0)
    m = values.as_mut_span()
    p: *mut u8 = m.as_mut_ptr()
    unsafe:  # SAFETY: both bytes are 0 or 1, the only bit patterns a `bool` may hold.
        q = p as *mut bool
        println(read(q, 0), read(q, 1))
