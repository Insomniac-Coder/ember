#$ test: run-fail
#$ rules: SPN-2, SPN-3
#$ panics: index 3 is out of bounds for a length of 2

# A split boundary may equal the length, but it may not exceed it. This is an
# ordinary bounds check before either result view is constructed.

fn main():
    values: Array[i32] = Array[i32]()
    values.push(1)
    values.push(2)
    span = values.as_span()
    _parts = span.split_at(3)
