#$ test: run-pass
#$ rules: RT-4
#$ profiles: debug, development, release
#$ stdout: 3
# `[RT-4]` — the table a backtrace names functions by is strings the program carries outside
# `release`; a program that does not panic only carries it.

fn add(rhs: int, b: int) -> int:
    return rhs + b

fn main():
    println(add(1, 2))
