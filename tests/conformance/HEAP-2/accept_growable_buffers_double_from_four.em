#$ test: run-pass
#$ rules: HEAP-2
#$ profiles: debug, development, release
#$ stdout: 0
#$ 4
#$ 5 8
#$ 9 16
## `[HEAP-2]` — growable buffers double, from a minimum of four elements.

fn main():
    xs: Array[int] = []
    println(xs.capacity())
    xs.push(owns 1)
    println(xs.capacity())
    for i in range(4):
        xs.push(owns i)
    println(xs.len(), xs.capacity())
    for i in range(4):
        xs.push(owns i)
    println(xs.len(), xs.capacity())
