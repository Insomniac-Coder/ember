#$ test: run-pass
#$ rules: SPN-1
## "`Array[T]` coerces to `Span[T]` at borrow sites and to `MutSpan[T]` at
## `mut` sites; `[T; N]` likewise."

fn total(xs: Span[i32]) -> i32:
    sum: i32 = 0
    i: usize = 0
    while i < xs.len():
        sum = sum + xs[i]
        i = i + 1
    return sum

fn fill(mut xs: MutSpan[i32], v: i32):
    i: usize = 0
    while i < xs.len():
        xs[i] = v
        i = i + 1

fn main():
    a: Array[i32] = Array[i32]()
    a.push(3)
    a.push(4)
    a.push(5)
    println(total(a))
    fixed: [i32; 3] = [1, 2, 3]
    println(total(fixed))
    fill(a, 9)
    println(total(a))
#$ stdout: 12
#$ 6
#$ 27
