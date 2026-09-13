fn main():
    values: Array[i32] = Array[i32]()
    values.push(1)
    values.push(2)
    for value in values:
        values.push(3)
        println(value)
