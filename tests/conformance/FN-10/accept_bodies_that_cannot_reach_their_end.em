#$ test: run-pass
#$ rules: FN-10
#$ profiles: debug, release, shipping
#$ stdout: 1
#$ -1
#$ 7
#$ 3
# A body whose every path returns or loops forever needs no value at its end:
# `if`/`else` and `match` whose branches all return, and a `while true` with
# no `break`.

enum Dir:
    Up
    Down

fn sign(x: int) -> int:
    if x >= 0:
        return 1
    else:
        return -1

fn step(d: Dir) -> int:
    match d:
        Up => return 7
        Down => return -7

fn first_multiple_of_three(start: int) -> int:
    n = start
    while true:
        if n % 3 == 0:
            return n
        n += 1

fn main():
    println(sign(5))
    println(sign(-5))
    println(step(Dir.Up))
    println(first_multiple_of_three(1))
