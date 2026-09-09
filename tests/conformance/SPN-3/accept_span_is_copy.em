#$ test: run-pass
#$ rules: SPN-3
## "`Span[T]` is `Copy`" — two live views of the same data are two readers,
## which `[BRW-1]` permits.

fn total(a: Span[i32], b: Span[i32]) -> i32:
    return a[0] + b[0]

fn main():
    xs: Array[i32] = Array[i32]()
    xs.push(21)
    v: Span[i32] = xs
    println(total(v, v))
#$ stdout: 42
