#$ test: run-pass
#$ rules: SPN-2, SPN-3, BRW-5
#$ stdout: 1
#$ stdout: 2
#$ stdout: 3
#$ stdout: 0
#$ stdout: 3
#$ stdout: 2

# A shared split preserves the source region and permits the boundary at both
# ends. The boundary element belongs to the right half.

fn left_len[T](span: Span[T], boundary: int) -> int:
    parts = span.split_at(boundary)
    return parts.0.len()

fn main():
    values: Array[i32] = Array[i32]()
    values.push(1)
    values.push(2)
    values.push(3)
    whole = values.as_span()
    parts = whole.split_at(1)
    left = parts.0
    right = parts.1
    println(left[0])
    println(right[0])
    println(right[1])

    end_parts = whole.split_at(3)
    tail = end_parts.1
    println(tail.len())
    println(whole.len())
    println(left_len[i32](whole, 2))
