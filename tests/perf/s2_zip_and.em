#$ perf: scalar
## `[TST-28]` — `bench/programs/s2_zip_and.em` and its twin, gated.

fn main():
    n = 1000000
    xs: Array[int] = []
    ys: Array[int] = []
    for i in 0..n:
        xs.push(i % 1000)
        ys.push(i % 777)
    for round in 0..300:
        for (a, b) in xs.iter_mut().zip(ys.iter()):
            a ^= b ^ round
    total = 0
    for x in xs:
        total ^= x
    println(total)
