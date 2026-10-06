fn main():
    n = 100000
    xs: Array[float] = []
    vs: Array[float] = []
    for i in 0..n:
        xs.push(i as float)
        vs.push(0.5)
    for step in 0..5000:
        for i in 0..n:
            xs[i] = xs[i] + vs[i]
    total = 0.0
    for x in xs:
        total = total + x
    println(total as int)
