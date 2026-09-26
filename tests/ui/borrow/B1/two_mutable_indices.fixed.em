fn main():
    values: Array[i32] = Array[i32]()
    values.push(1)
    values.push(2)
    parts = values.as_mut_span().split_at(1)
    left = parts.0
    right = parts.1
    first: ref mut i32 = ref mut left[0]
    second: ref mut i32 = ref mut right[0]
    first = 10
    second = 20
