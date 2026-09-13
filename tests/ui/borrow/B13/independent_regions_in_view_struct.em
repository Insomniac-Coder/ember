@view
struct Pair:
    a: Span[i32]
    b: Span[i32]

fn bundle(x: Span[i32], y: Span[i32]) -> Pair:
    return Pair(x, y)

fn first(pair: Pair) -> i32:
    return pair.a[0]

fn main():
    long: Array[i32] = Array[i32]()
    long.push(1)
    short: Array[i32] = Array[i32]()
    short.push(2)
    pair = bundle(long, short)
    short.push(3)
    println(first(pair))

