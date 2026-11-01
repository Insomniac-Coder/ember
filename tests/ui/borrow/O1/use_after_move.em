fn take(owns xs: Array[i32]) -> i32:
    return xs[0]

fn main():
    values: Array[i32] = Array[i32]()
    values.push(owns 1)
    println(take(owns values))
    println(take(owns values))

