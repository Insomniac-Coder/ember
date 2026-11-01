@view
struct Pair:
    first: Span[int]
    second: Span[int]

fn keep(a: Span[int], b: Span[int]) -> Pair:
    return Pair(a, b)

fn swap(a: Span[int], b: Span[int]) -> Pair:
    return Pair(b, a)

fn call(make: fn(Span[int], Span[int]) -> Pair, a: Span[int], b: Span[int]) -> Pair from a:
    return make(a, b)

fn main():
    xs = [1, 2]
    ys = [3]
    make = keep if input() == "yes" else swap
    println(call(make, xs, ys).first)
