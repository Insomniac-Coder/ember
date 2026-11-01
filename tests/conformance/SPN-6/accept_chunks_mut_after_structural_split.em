#$ test: run-pass
#$ rules: SPN-6, BRW-5, SPN-3, TST-25

fn main():
    values: Array[i32] = Array[i32]()
    values.push(owns 1)
    values.push(owns 2)
    values.push(owns 3)
    values.push(owns 4)
    halves = values.as_mut_span().split_at(2)
    left = halves.0
    right = halves.1
    left_chunks = left.chunks_mut(1)
    right_chunks = right.chunks_mut(1)
    left_first = left_chunks.next()
    right_first = right_chunks.next()
    match owns left_first:
        Some(chunk):
            chunk[0] = 10
        None:
            pass
    match owns right_first:
        Some(chunk):
            chunk[0] = 30
        None:
            pass
    println(left[0])
    println(right[0])
#$ stdout: 10
#$ stdout: 30
