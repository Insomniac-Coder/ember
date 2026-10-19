#$ test: compile-fail
#$ rules: DET-2, DET-1
#$ error[E4070]: `@deterministic` function `address` reaches a nondeterministic operation
# `[DET-2]` — an address observed as an integer differs from run to run: it is `Nondet`.

@deterministic
fn address(p: *mut i32) -> usize:
    unsafe:  # SAFETY: only the address is taken, as a number
        return p as usize

fn main():
    p: *mut i32
    unsafe:  # SAFETY: one slot, freed below; only its address is read
        p = alloc[i32](1)
    println(address(p) > 0)
    unsafe:  # SAFETY: the slot allocated above, freed once
        free(p, 1)
