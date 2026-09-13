fn main():
    values: Array[i32] = Array[i32]()
    values.push(1)
    values.push(2)
    count = values.len()
    for index in 0..count:
        value = values[index]
        values.push(3)
        println(value)
