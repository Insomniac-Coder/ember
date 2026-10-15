#$ test: run-pass
#$ rules: EFF-23, EFF-12
#$ stdout: 6
# `[EFF-23]` — only a contract whose checking is not built is refused, and every contract's is
# built now: `@static_safe` here is accepted and checked, and its bounds checks are allowed
# (`[EFF-12]`).

@static_safe
fn total(xs: Span[int]) -> int:
    sum = 0
    for i in 0..xs.len():
        sum += xs[i]
    return sum

fn main():
    values = [1, 2, 3]
    println(total(values.as_span()))
