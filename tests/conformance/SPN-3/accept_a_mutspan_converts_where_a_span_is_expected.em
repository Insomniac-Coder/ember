#$ test: run-pass
#$ rules: SPN-3, TYP-5, SPN-1
#$ stdout: 6
#$ stdout: 1
#$ stdout: 15
#$ stdout: 9
#$ stdout: 30
# `[SPN-3]`, `[TYP-5]` rule 6 (ODR-116) — a `MutSpan[T]` where a `Span[T]` is expected converts, a
# shared reborrow: a variable, a generic parameter's `Span[T]`, a view made in the argument, and a
# `MutSpan` after the `Span` is done with may be written again.

fn total(xs: Span[i32]) -> i32:
    t: i32 = 0
    for x in xs:
        t += x
    return t

fn first[T: Copy](xs: Span[T]) -> T:
    return xs[0]

fn main():
    a: Array[i32] = [1, 2, 3]
    m: MutSpan[i32] = a.as_mut_span()
    println(total(m))
    println(first(m))
    m[0] = 10
    println(total(m))
    b: Array[i32] = [4, 5]
    println(total(b.as_mut_span()))
    view: Span[i32] = m
    println(total(view) * 2)
