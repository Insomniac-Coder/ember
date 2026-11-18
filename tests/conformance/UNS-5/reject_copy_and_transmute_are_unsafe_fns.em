#$ test: compile-fail
#$ rules: UNS-5, UNS-1
# `[UNS-5]` — `transmute`, `copy` and `copy_nonoverlapping` are `unsafe fn`s: a call of one outside
# unsafe code is `E3100`; a copy's message names the raw pointer it writes through and what must be
# writable there.

from std.mem import copy, copy_nonoverlapping, transmute

fn main():
    a: Array[i32] = [1, 2, 3, 4]
    b: Array[i32] = [0, 0, 0, 0]
    p = a.as_span().as_ptr()
    q = b.as_mut_span().as_mut_ptr()
    copy_nonoverlapping(p, q, 4)    #$ error[E3100]: `copy_nonoverlapping(p, q, 4)` (line 14) writes through the raw pointer `q`, and nothing checks that four `i32`s are writable there; put line 14 in an `unsafe:` block with a `# SAFETY:` note saying why that holds here
    n = b.len()
    copy(p, q, n)    #$ error[E3100]: `copy(p, q, n)` (line 16) writes through the raw pointer `q`, and nothing checks that `n` `i32`s are writable there
    f = transmute[u32, f32](owns 1)    #$ error[E3100]: `transmute[u32, f32](owns 1)` (line 17) calls an `unsafe fn`, whose callers must make true what its `@safety` says
    println(f)
