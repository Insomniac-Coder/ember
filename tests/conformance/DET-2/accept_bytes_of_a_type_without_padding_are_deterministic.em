#$ test: run-pass
#$ rules: DET-2, DET-1
#$ stdout: 1
# `[DET-2]` — a `u32` has no padding: reading its bytes through a byte pointer reads only what was
# written, so a `@deterministic` function may do it (the byte order is the target's).

@deterministic
fn low_byte(p: *mut u32) -> u8:
    unsafe:  # SAFETY: a u32 is four bytes; byte 0 exists
        bytes = p as *mut u8
        return read(bytes, 0)

fn main():
    p: *mut u32
    unsafe:  # SAFETY: one slot, written before it is read
        p = alloc[u32](1)
        write(p, 0, 1 as u32)
    println(low_byte(p))
    unsafe:  # SAFETY: the slot allocated above, freed once
        free(p, 1)
