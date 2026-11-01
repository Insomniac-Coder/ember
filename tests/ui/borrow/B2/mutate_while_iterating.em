fn main():
    values: Array[i32] = Array[i32]()
    values.push(owns 1)
    values.push(owns 2)
    for value in values:
        values.push(owns 3)
        println(value)
