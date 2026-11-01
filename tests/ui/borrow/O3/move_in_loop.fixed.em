fn consume(owns xs: Array[i32]) -> i32:
    return xs[0]

fn main():
    i: i32 = 0
    while i < 2:
        values: Array[i32] = Array[i32]()
        values.push(owns 1)
        println(consume(owns values))
        i = i + 1

