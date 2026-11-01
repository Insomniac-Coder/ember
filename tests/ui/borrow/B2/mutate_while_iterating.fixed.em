fn main():
    values: Array[i32] = Array[i32]()
    values.push(owns 1)
    values.push(owns 2)
    count = values.len()
    for index in 0..count:
        value = values[index]
        values.push(owns 3)
        println(value)
