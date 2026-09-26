#$ test: run-fail
#$ rules: BRW-5, SPN-2
#$ panics: index 2 is out of bounds for a length of 1

# A split boundary may equal the length, but it may not exceed it. The check
# is an ordinary visible MIR bounds assertion, not a hidden backend condition.

fn main():
    values: Array[i32] = Array[i32]()
    values.push(1)
    _parts = values.as_mut_span().split_at(2)
