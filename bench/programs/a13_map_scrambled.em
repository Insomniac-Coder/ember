@overflow(wrap)
fn key(i: int) -> int:
    return (i * 0x5851F42D4C957F2D) >> 8

fn main():
    m: Map[int, int] = {}
    for i in 0..1000000:
        m.insert(key(i), i)
    total = 0
    for round in 0..5:
        for i in 0..1000000:
            total = total + m[key(i)]
    println(len(m), total)
