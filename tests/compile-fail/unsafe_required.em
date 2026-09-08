#$ test: compile-fail
#$ rules: UNS-1
#$ error[E3100]: `alloc` needs an `unsafe` block
#$ error[E3100]: `write` needs an `unsafe` block
#$ error[E3100]: `read` needs an `unsafe` block
#$ error[E3100]: `free` needs an `unsafe` block

# `[UNS-1]` â€” the raw primitives are not reachable from safe code. Each of
# these four is a separate error; `size_of` on the last line is not, because
# it reads no memory.
fn main():
    p: *mut i32 = alloc[i32](4)
    write(p, 0, 1)
    println(read(p, 0))
    free(p, 4)
    println(size_of[i32]())
