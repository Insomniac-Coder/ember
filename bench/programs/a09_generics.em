fn larger[T: Ord](a: T, b: T) -> T:
    if a > b:
        return a
    return b

fn main():
    best = 0
    total = 0
    for i in 0..200000000:
        best = larger(best, (i * 7919) % 100003)
        total = total + best
    println(best, total)
