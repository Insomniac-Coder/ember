#$ test: run-pass
#$ rules: HEAP-8, HEAP-2
#$ profiles: debug, release, shipping
#$ stdout: true
#$ stdout: 5 8
# `[HEAP-8]` — a capacity whose bytes fit is granted as asked: `reserve(n)`
# holds at least `n` more, and `extend` makes room for what it appends.

fn main():
    xs: Array[int] = []
    xs.reserve(1000)
    println(xs.capacity() >= 1000)
    for i in 0..3:
        xs.push(i)
    ys: Array[int] = [7, 8]
    xs.extend(ys)
    println(xs.len(), xs[4])
