fn main():
    n = 100000
    a: Array[int] = []
    b: Array[int] = []
    out: Array[int] = []
    for i in 0..n:
        a.push(i)
        b.push(3 * i)
        out.push(0)
    for round in 0..2000:
        for i in 0..n:
            out[i] = a[i] + b[i] + round
    total = 0
    for i in 0..n:
        total = total + out[i]
    println(total)
