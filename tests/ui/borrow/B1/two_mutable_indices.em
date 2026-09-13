fn main():
    values: Array[i32] = Array[i32]()
    values.push(1)
    values.push(2)
    i: usize = 0
    j: usize = 1
    first: ref mut i32 = ref mut values[i]
    second: ref mut i32 = ref mut values[j]
    first = 10
    second = 20
