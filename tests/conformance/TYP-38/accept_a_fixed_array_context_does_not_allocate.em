#$ test: run-pass
#$ rules: TYP-38, SPN-1
#$ profiles: debug, release, shipping
#$ stdout: 6
#$ 15
#$ assert-c: !contains("ember_vec_from_elems")
# Where a `[T; N]` or a `Span[T]` is expected, a list literal is a fixed array
# (and a view of a temporary one) and allocates nothing.

fn total(xs: Span[int]) -> int:
    sum = 0
    for i in 0..xs.len():
        sum += xs[i]
    return sum

fn main():
    fixed: [int; 3] = [1, 2, 3]
    println(total(fixed))
    println(total([4, 5, 6]))
