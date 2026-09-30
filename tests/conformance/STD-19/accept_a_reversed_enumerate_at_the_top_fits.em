#$ test: run-pass
#$ rules: STD-19, CTL-3b
#$ profiles: debug, release, shipping
#$ stdout: 9223372036854775807 2
#$ stdout: 9223372036854775806 1
#$ stdout: [(9223372036854775807, 2), (9223372036854775806, 1)]
# ODR-091 — numbers that reach `int`'s top and no further run backwards with
# no panic, in a `for` (a counted loop) and through the library (`to_array`).

fn main():
    xs: Array[int] = [1, 2]
    for i, x in xs.iter().enumerate(9223372036854775806).rev():
        println(i, x)
    pairs: Array[(int, int)] = xs.iter().copied().enumerate(9223372036854775806).rev().to_array()
    println(pairs)
