#$ test: run-pass
#$ rules: BRW-5, SPN-3
#$ stdout: 10
#$ stdout: 20
#$ stdout: 0
#$ stdout: 0

# `split_at_mut` creates one mutable borrow of the Array and returns two
# non-overlapping mutable views. The boundary element belongs to the right
# half, and writes through both halves reach the original buffer.

fn main():
    values: Array[i32] = Array[i32]()
    values.push(1)
    values.push(2)
    parts = values.split_at_mut(1)
    left = parts.0
    right = parts.1
    left[0] = 10
    right[0] = 20
    println(left[0])
    println(right[0])

    empty: Array[i32] = Array[i32]()
    empty_parts = empty.split_at_mut(0)
    empty_left = empty_parts.0
    empty_right = empty_parts.1
    println(empty_left.len())
    println(empty_right.len())
