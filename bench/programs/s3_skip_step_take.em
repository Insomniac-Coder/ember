fn main():
    n = 1000000
    xs: Array[int] = []
    for i in 0..n:
        xs.push(i % 1000)
    total = 0
    for round in 0..300:
        for x in xs.iter().skip(round).step_by(3).take(300000):
            total ^= x
    println(total)
