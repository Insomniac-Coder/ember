#$ test: run-pass
#$ rules: RT-12
#$ stdout: 5000
# `[RT-12]` — the handler changes nothing for a program whose stack fits: a recursion 5000 deep runs
# to its end.

fn count(n: int) -> int:
    if n == 0:
        return 0
    return count(n - 1) + 1

fn main():
    println(count(5000))
