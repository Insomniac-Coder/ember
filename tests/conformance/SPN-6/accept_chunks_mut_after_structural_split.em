#$ test: run-pass
#$ rules: SPN-6, BRW-5, SPN-3, TST-25

fn main():
    values: Array[i32] = Array[i32]()
    values.push(1)
    values.push(2)
    values.push(3)
    values.push(4)
    halves = values.split_at_mut(2)
    left = halves.0
    right = halves.1
    left_chunks = left.chunks_mut(1)
    right_chunks = right.chunks_mut(1)
    left_first = left_chunks.next()
    right_first = right_chunks.next()
    match left_first:
        Some(chunk):
            chunk[0] = 10
        None:
            pass
    match right_first:
        Some(chunk):
            chunk[0] = 30
        None:
            pass
    println(left[0])
    println(right[0])
#$ stdout: 10
#$ stdout: 30
