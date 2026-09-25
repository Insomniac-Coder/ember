#$ test: run-pass
#$ rules: EXP-1, STD-9
#$ stdout: 1 2
#$ stdout: [3] 1
# `[EXP-1]` — a `Copy` argument of `println` is read where it is written, so a
# later argument that changes the variable does not change what is printed.

fn bump(mut n: int) -> int:
    n += 1
    return n

fn main():
    n = 1
    println(n, bump(n))
    ys = [3]
    println(ys, len(ys))
