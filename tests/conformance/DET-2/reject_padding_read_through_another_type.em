#$ test: compile-fail
#$ rules: DET-2, DET-1
#$ error[E4070]: `@deterministic` function `first_bytes` reaches a nondeterministic operation
# `[DET-2]` — `Pair` has three padding bytes after `tag`: reinterpreting a pointer to it as a pointer
# to bytes lets them be read, and they hold nothing defined. (ODR-136: recorded at the cast.)

struct Pair:
    tag: u8
    value: u32

@deterministic
fn first_bytes(p: *mut Pair) -> u8:
    unsafe:  # SAFETY: the pair is four bytes and more; byte 0 is `tag`
        bytes = p as *mut u8
        return read(bytes, 0)

fn main():
    p: *mut Pair
    unsafe:  # SAFETY: one pair, written before it is read
        p = alloc[Pair](1)
        write(p, 0, Pair(tag = 7, value = 9))
    println(first_bytes(p))
    unsafe:  # SAFETY: the pair allocated above, freed once
        free(p, 1)
