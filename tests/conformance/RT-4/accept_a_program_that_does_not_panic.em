#$ test: run-pass
#$ rules: RT-4
#$ profiles: debug, development, release
#$ stdout: 3
# `[RT-4]` — the table a backtrace names functions by is strings the program carries outside
# `release`; a program that does not panic only carries it.

fn add(a: int, b: int) -> int:
    return a + b

fn main():
    println(add(1, 2))
