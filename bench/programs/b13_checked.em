fn main():
    n = 100000
    a: Array[int] = []
    out: Array[int] = []
    for i in 0..n:
        a.push(i % 1000)
        out.push(0)
    for round in 0..2000:
        for i in 0..n:
            out[i] = (out[i] + a[i]) & 1023
    total = 0
    for i in 0..n:
        total = total + out[i]
    println(total)
