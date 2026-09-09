#$ test: compile-fail
#$ rules: BRW-6, BRW-1
# `[BRW-6]`'s first clause: `ref r.f` is a shared reborrow that freezes `r`
# for its duration (scalar whole-place form here). Writing the owner while it
# is live is `E3021` (B3), the same shape `compile-fail/` pins for derived
# borrows — the loan here is already shared, so no reservation is involved.

fn main():
    x: i32 = 1
    r: ref mut i32 = ref mut x
    s: ref i32 = ref r
    x = 2                 #$ error[E3021]: `x` cannot be written while it is borrowed
    println(s)
