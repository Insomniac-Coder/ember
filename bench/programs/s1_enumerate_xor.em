fn main():
    n = 1000000
    xs: Array[int] = []
    for i in 0..n:
        xs.push(i % 1000)
    for round in 0..300:
        for (i, x) in xs.iter_mut().enumerate(start=round):
            x ^= i
    total = 0
    for x in xs:
        total ^= x
    println(total)
