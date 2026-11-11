#$ test: run-pass
#$ rules: HEAP-1
#$ stdout: 1
## `[HEAP-1]` — the same growth in ordinary code allocates through the runtime's allocator.

fn grow(ref mut xs: Array[int]):
    xs.push(owns 1)

fn main():
    xs: Array[int] = []
    grow(ref mut xs)
    println(xs.len())
