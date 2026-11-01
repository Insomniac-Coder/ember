fn main():
    m: Map[String, int] = {}
    for i in 0..200000:
        m.insert(owns f"key{i}", owns i)
    total = 0
    for round in 0..5:
        for i in 0..200000:
            total = total + m[f"key{i}"]
    println(len(m), total)
