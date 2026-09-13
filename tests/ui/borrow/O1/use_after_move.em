fn take(owned xs: Array[i32]) -> i32:
    return xs[0]

fn main():
    values: Array[i32] = Array[i32]()
    values.push(1)
    println(take(values))
    println(take(values))

