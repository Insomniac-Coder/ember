#$ test: compile-pass
#$ rules: LNT-1, DIA-14
# `[LNT-1]` — a binding the source never reads is reported once, at its
# declaration, however many instances the generic function has.

fn f[T](x: T) -> int:
    k = 1  #$ warning[L1001]: `k` is never read
    return 2

fn main():
    println(f(1), f("a"))
