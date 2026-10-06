fn main():
    m: Map[int, int] = {}
    for i in 0..1000000:
        m.insert(i * 7, i)
    total = 0
    for round in 0..5:
        for i in 0..1000000:
            total = total + m[i * 7]
    println(len(m), total)
