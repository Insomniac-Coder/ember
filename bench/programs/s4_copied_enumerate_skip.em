fn main():
    n = 1000000
    xs: Array[int] = []
    for i in 0..n:
        xs.push(i % 1000)
    total = 0
    for round in 0..300:
        for (i, x) in xs.iter().copied().enumerate().skip(round):
            total ^= i | x
    println(total)
