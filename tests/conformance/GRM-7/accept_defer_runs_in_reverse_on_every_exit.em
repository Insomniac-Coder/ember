#$ test: run-pass
#$ rules: GRM-7
#$ stdout: iter 0
#$ stdout: iter 1
#$ stdout: iter 2
#$ stdout: second
#$ stdout: outer
#$ stdout: 3
## `[GRM-7]` — `defer` blocks run in reverse order at the exit of their
## block, on every path out: the end of an iteration, `continue`, `break` and
## `return`.

fn work(n: int) -> int:
    defer:
        println("outer")
    for i in 0..n:
        defer:
            println(f"iter {i}")
        if i == 1:
            continue
        if i == 2:
            break
    defer:
        println("second")
    return n

fn main():
    println(work(3))
