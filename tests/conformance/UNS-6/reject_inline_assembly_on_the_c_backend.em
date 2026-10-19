#$ test: compile-fail
#$ rules: UNS-6
#$ error[E5090]: inline assembly is not supported by the C backend
# `[UNS-6]` — inline assembly is `unsafe asm("…", …)` where the toolchain supports it, and `E5090`
# otherwise: the C backend writes portable C, which has none.

fn main():
    unsafe:  # SAFETY: a test of the refusal
        asm("nop")
