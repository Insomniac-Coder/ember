# Clause BRW-5.4: "`get_pair_mut(i, j) -> Option[(ref mut T, ref mut T)]`, on an `Array` or a
# `MutSpan`, views the container once, checks that both indices are in range and differ, and only
# then makes the two views; otherwise it is `None` and the container is unchanged." BRW-5.5:
# "Distinct elements of a zero-sized type need not have distinct addresses, and nothing about
# addresses follows from the indices (ODR-068)."
#$ test: run-pass
#$ rules: BRW-5, STD-15, SPN-5, GRM-44
#$ stdout: [1, 5, 3, 2]
#$ stdout: false false false
#$ stdout: [1, 5, 3, 2]
#$ stdout: 2
# On a `MutSpan` and on an `Array`; equal indices and an index out of range give `None` and change
# nothing; for a type with no fields the indices alone decide.

struct Tag:
    pass

fn bump_pair(ref mut view: MutSpan[int]):
    if view.get_pair_mut(3, 1) exists as (ref a, ref mut b):
        b += a

fn main():
    xs = [1, 3, 3, 2]
    bump_pair(ref mut xs)
    println(xs)
    same = xs.get_pair_mut(1, 1).is_some()
    out = xs.get_pair_mut(0, 4).is_some()
    below = xs.get_pair_mut(-1, 0).is_some()
    println(same, out, below)
    println(xs)
    tags = [Tag(), Tag(), Tag()]
    n = 0
    if tags.get_pair_mut(0, 2).is_some():
        n += 1
    if tags.get_pair_mut(2, 1).is_some():
        n += 1
    if tags.get_pair_mut(1, 1).is_some():
        n += 1
    println(n)
