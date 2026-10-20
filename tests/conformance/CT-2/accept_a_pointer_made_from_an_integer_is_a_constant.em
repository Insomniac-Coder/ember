#$ test: run-pass
#$ rules: CT-2
#$ profiles: debug, development, release
#$ stdout: 0 18446744073709551615 4096 true
# `[CT-2]` (ODR-145) — a raw pointer made from an integer is a compile-time value: an address, read
# by nothing, so a `const` of pointer type holds one, and making it asks no `unsafe` there. The
# integer's low bits at the pointer's width are the address, as the cast makes them at run time.

const NIL: *mut void = 0 as *mut void
const ALL: *mut void = -1 as *mut void
const PAGE: *i32 = 4096 as *i32

fn main():
    unsafe:  # SAFETY(ffi): the addresses are printed and compared, never read
        println(NIL as usize, ALL as usize, PAGE as usize, NIL == 0 as *mut void)
