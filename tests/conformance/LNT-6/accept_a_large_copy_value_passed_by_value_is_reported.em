#$ test: run-pass
#$ command: lint
#$ rules: LNT-6, BRW-8
#$ stdout: 2 2 1
## `[LNT-6]` — `L2002`: an `owned` parameter of a `Copy` type larger than 256 bytes, the default
## `large_copy` threshold, is copied in by every call; a borrowed one is passed by address
## (`[BRW-8]`), and a small one is cheap to copy.

@derive(Copy)
struct Samples:
    values: [f64; 64]
    count: int

@derive(Copy)
struct Pair:
    a: int
    b: int

fn by_value(owned s: Samples) -> int:  #$ warning[L2002]: `s` is a 520-byte `Copy` value passed by value
    return s.count

fn by_address(s: Samples) -> int:
    return s.count

fn small(owned p: Pair) -> int:
    return p.a

fn main():
    s = Samples([0.0; 64], 2)
    println(by_value(s), by_address(s), small(Pair(1, 2)))
