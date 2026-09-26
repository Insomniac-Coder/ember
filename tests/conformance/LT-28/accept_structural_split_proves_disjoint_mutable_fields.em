#$ test: run-pass
#$ rules: LT-18, LT-24, LT-27, LT-28, BRW-5, TST-17
#$ stdout: 10
#$ stdout: 20

# Distinct region slots are not themselves a no-alias proof. A mutable
# view's `split_at` supplies the existing structural proof, so its two disjoint result views may
# become the independently mutable fields of one multi-region view.

@view
struct MutPair:
    left: MutSpan[i32]
    right: MutSpan[i32]

fn main():
    values: Array[i32] = Array[i32]()
    values.push(1)
    values.push(2)
    parts = values.as_mut_span().split_at(1)
    pair = MutPair(parts.0, parts.1)
    pair.left[0] = 10
    pair.right[0] = 20
    println(pair.left[0])
    println(pair.right[0])
