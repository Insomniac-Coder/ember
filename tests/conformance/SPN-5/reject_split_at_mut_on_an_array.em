#$ test: compile-fail
#$ rules: SPN-5, STD-15
#$ help: `.as_mut_span().split_at(k)` gives two `MutSpan`s
# `[SPN-5]` (F-075, D-351) — there is no `split_at_mut`: the mutable split is
# `split_at` of the array's mutable view. The compiler used to accept an
# `Array.split_at_mut` of its own; the error names the spelling there is.

fn main():
    xs = [1, 2, 3]
    halves = xs.split_at_mut(1) #$ error[E1010]: `Array[i64]` has no method named `split_at_mut`
    (left, right) = xs.as_mut_span().split_at(1)
    left[0] = right[0]
    println(xs)
