#$ test: compile-fail
#$ rules: LT-18, LT-27, LT-28, BRW-1, BRW-5, TST-17

# Independent region slots never assert that their storage is disjoint. The
# second `as_mut_span` therefore remains an ordinary overlapping mutable
# borrow, rather than becoming legal merely because `MutPair` has two fields.

@view
struct MutPair:
    left: MutSpan[i32]
    right: MutSpan[i32]

fn main():
    values: Array[i32] = Array[i32]()
    values.push(1)
    left = values.as_mut_span()
    right = values.as_mut_span() #$ error[E3022]: `values` is already mutably borrowed
    pair = MutPair(left, right)
    println(pair.left[0])
