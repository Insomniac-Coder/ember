#$ test: run-pass
#$ rules: ARN-3, ARN-11, ARN-12, TST-23
# The Phase-2 core predicate proves scalar zero validity without depending on
# general derive support. `alloc_array` returns initialized, mutable elements.

fn main():
    arena = Arena.with_capacity(64)
    values: MutSpan[i32] = arena.alloc_array[i32](3)
    println(values[0])
    println(values[2])
    values[1] = 7
    println(values[1])
#$ stdout: 0
#$ 0
#$ 7
