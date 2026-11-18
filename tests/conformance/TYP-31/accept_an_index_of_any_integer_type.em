#$ test: run-pass
#$ rules: TYP-31
#$ stdout: 20 20 20 20 20 20
## `[TYP-31]` - indexing accepts an index of any integer type.

fn main():
    xs = [10, 20, 30]
    a: i8 = 1
    b: u16 = 1
    c: i64 = 1
    d: u64 = 1
    e: i128 = 1
    f: u128 = 1
    println(xs[a], xs[b], xs[c], xs[d], xs[e], xs[f])
