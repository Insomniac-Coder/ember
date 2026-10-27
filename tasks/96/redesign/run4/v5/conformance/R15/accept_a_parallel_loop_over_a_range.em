#$ test: run-pass
#$ rules: THR-1, FN-1
#$ stdout: [2, 0, 1]
## design rules: R15, R4
## R15: a `@parallel` loop over a range, each chunk writing only its own `xs[i]` (AN-84's `place`).

fn place(ref mut xs: MutSpan[int], idx: Span[int]):
    @parallel
    for i in 0..idx.len():
        xs[i] = idx[i]

fn main():
    a = [0, 0, 0]
    order = [2, 0, 1]
    place(ref mut a.as_mut_span(), order.as_span())
    println(a)
