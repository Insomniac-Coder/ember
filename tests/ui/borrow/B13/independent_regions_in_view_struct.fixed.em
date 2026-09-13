fn first(x: Span[i32], y: Span[i32]) -> i32:
    println(y[0])
    return x[0]

fn main():
    long: Array[i32] = Array[i32]()
    long.push(1)
    short: Array[i32] = Array[i32]()
    short.push(2)
    println(first(long, short))
    short.push(3)

