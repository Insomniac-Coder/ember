#$ test: compile-fail
#$ rules: SPN-3, TYP-5, BRW-2
# `[SPN-3]` (ODR-116) — the `Span` a `MutSpan` converts to is a shared reborrow of it: the `MutSpan`
# is not written while the `Span` lives.

fn main():
    a: Array[i32] = [1, 2, 3]
    m: MutSpan[i32] = a.as_mut_span()
    view: Span[i32] = m
    m[0] = 10              #$ error[E3021]: `m` cannot be written while it is borrowed
    println(view[0])
