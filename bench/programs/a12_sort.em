fn main():
    xs: Array[int] = []
    for i in 0..5000000:
        xs.push((i * 7919) % 1000003)
    xs.sort()
    println(xs[0], xs[2500000], xs[4999999])
