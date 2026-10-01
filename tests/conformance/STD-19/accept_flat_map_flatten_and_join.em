#$ test: run-pass
#$ rules: STD-19, CTL-1
#$ stdout: [1, 10, 2, 20, 3, 30]
#$ stdout: [1, 2, 3, 4]
#$ stdout: 6
#$ stdout: 1, 2, 3
#$ stdout: x-y
# `[STD-19]` — `flat_map(f)` gives the items of each iterable `f` makes, a
# range included (D-444); `flatten()` the items of each item; `join(sep)` the
# items as text. D-407 had kept all three out of the library.

fn main():
    xs = [1, 2, 3]
    pairs: Array[int] = xs.iter().copied().flat_map(fn(x: int) => [x, x * 10]).collect()
    println(pairs)
    nested = [[1], [2, 3], [], [4]]
    flat: Array[int] = nested.iter().cloned().flatten().collect()
    println(flat)
    println(xs.iter().copied().flat_map(fn(x: int) => 0..x).count())
    println(xs.iter().copied().join(", "))
    words = ["x".to_string(), "y".to_string()]
    println(words.iter().cloned().join("-"))
