fn main():
    n = 2000000
    total = 0
    for round in 0..300:
        for (i, v) in (round..n).step_by(2).enumerate():
            total ^= i ^ v
    println(total)
