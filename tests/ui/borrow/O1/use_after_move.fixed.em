fn inspect(xs: Array[i32]) -> i32:
    return xs[0]

fn main():
    values: Array[i32] = Array[i32]()
    values.push(1)
    println(inspect(values))
    println(inspect(values))

