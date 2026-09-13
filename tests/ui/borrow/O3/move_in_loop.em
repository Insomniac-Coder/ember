fn consume(owned xs: Array[i32]) -> i32:
    return xs[0]

fn main():
    values: Array[i32] = Array[i32]()
    values.push(1)
    i: i32 = 0
    while i < 2:
        println(consume(values))
        i = i + 1

