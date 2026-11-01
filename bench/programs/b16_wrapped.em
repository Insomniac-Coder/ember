@overflow(wrap)
fn main():
    n = 100000
    a: Array[int] = []
    b: Array[int] = []
    out: Array[int] = []
    for i in 0..n:
        a.push(owns (i % 1000))
        b.push(owns (i % 7))
        out.push(owns 0)
    for round in 0..20000:
        for i in 0..n:
            out[i] = (((a[i] ^ round) + b[i]) - 7) & 1023
    total = 0
    for i in 0..n:
        total = total + out[i]
    println(total)
