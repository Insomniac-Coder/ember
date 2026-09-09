#$ test: parse-pass
#$ rules: GRM-23
# `in` and `not in` are binary operators at comparison precedence, and
# `not in` is a single operator — `x not in xs` is `not_in(x, xs)` and never
# `(not x) in xs`. `[STD-8]`'s `Contains` bound is what makes them mean
# anything, and that is not implemented, so this is a `parse-pass`: the
# grammar has landed ahead of the semantics.

fn main():
    xs: Array[i32] = Array[i32]()
    a = 1 in xs
    b = 1 not in xs
    c = (1 in xs) and (2 not in xs)
    println(1)
