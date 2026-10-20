#$ test: compile-fail
#$ rules: CT-2
# `[CT-2]` (ODR-145) — only a `const`'s own pointer made from an integer is a compile-time value: a
# pointer the evaluator computes may be into its own memory, and may not leave it (`E6010`); and
# outside a `const` the cast is the run-time one, which needs `unsafe` (`[UNS-1]`).

fn address() -> *mut i32:
    unsafe:  # SAFETY(ffi): the address is never read
        return 8 as *mut i32

fn main():
    p = comptime(address())  #$ error[E6010]: a compile-time value cannot be a `*mut i32`
    q = 16 as *mut i32  #$ error[E3100]: casting between a raw pointer and an integer requires `unsafe`
