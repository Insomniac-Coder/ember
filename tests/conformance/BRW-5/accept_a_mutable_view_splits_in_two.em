#$ test: run-pass
#$ rules: BRW-5, SPN-3
#$ stdout: 10
#$ stdout: 20
#$ stdout: 0
#$ stdout: 0

# `as_mut_span().split_at(k)` creates one mutable borrow of the Array and
# returns two non-overlapping mutable views (`[SPN-5]`; there is no
# `split_at_mut`, D-351). The boundary element belongs to the right
# half, and writes through both halves reach the original buffer.

fn main():
    values: Array[i32] = Array[i32]()
    values.push(1)
    values.push(2)
    parts = values.as_mut_span().split_at(1)
    left = parts.0
    right = parts.1
    left[0] = 10
    right[0] = 20
    println(left[0])
    println(right[0])

    empty: Array[i32] = Array[i32]()
    empty_parts = empty.as_mut_span().split_at(0)
    empty_left = empty_parts.0
    empty_right = empty_parts.1
    println(empty_left.len())
    println(empty_right.len())
