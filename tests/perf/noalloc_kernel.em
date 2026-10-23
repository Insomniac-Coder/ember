#$ perf: scalar
#$ allocations: 1
## `[BEN-6]` — an `@noalloc` path, its allocations counted exactly: the kernel runs 4,000 times over
## one buffer and allocates nothing (the type checker proves it, `[EFF-2]`); the program makes one
## allocation, the buffer's, reserved whole before it is filled, as its C twin's one `malloc`.

@noalloc
@overflow(wrap)
fn mix(mut values: Array[int], seed: int) -> int:
    acc = seed
    for i in 0..values.len():
        v = values[i] * 31 + acc
        values[i] = v % 1000003
        acc = acc ^ v
    return acc

@overflow(wrap)
fn main():
    values: Array[int] = []
    values.reserve(100000)
    for i in 0..100000:
        values.push(i)
    total = 0
    for round in 0..4000:
        total = total + mix(values, round)
    println(total)
