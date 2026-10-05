#$ test: run-pass
#$ rules: STD-19
#$ stdout: 6 3 6 6
#$ stdout: [(-1, 0), (0, 0), (1, 1), (2, 0), (3, 1), (4, 2)]
#$ profiles: debug, release
# G8-4 (the owner's design, 0.9.10) — `flat_map` and `flatten` count every inner iterator's items
# together, stepping up by need: in the level that holds the outer count times the inner one. Stored
# items by stored items stay an `int`; an `int` by a `u64` (an array of ranges of 64-bit numbers)
# or a `u64` by a `u64` is a `u128`; `u256` is the limit. Their numbers follow from that count.

fn main():
    xs: Array[int] = [1, 2, 3]
    nested: Array[Array[int]] = [[1, 2], [3]]
    a: int = xs.iter().copied().flat_map(fn(x: int) => [x, x * 10]).count()
    c: int = nested.iter().cloned().flatten().count()
    outer = 0 as u64 .. 4 as u64
    b: u128 = outer.iter().flat_map(fn(i: u64) => 0 as u64 .. i).count()
    d: u128 = xs.iter().copied().flat_map(fn(x: int) => 0..x).count()
    println(a, c, b, d)
    println(outer.iter().flat_map(fn(i: u64) => 0 as u64 .. i).enumerate(-1).to_array())
