#$ test: run-fail
#$ rules: STD-15
#$ panics: drain range reaches outside the array
# `[STD-15]` (ODR-031) — an unsigned bound past `int`'s range is outside the
# array too: `0..=u64` maximum panics rather than drain nothing.

fn main():
    xs = [1, 2, 3]
    end: u64 = 18446744073709551615
    taken = xs.drain(0..=end)
    println(taken)
