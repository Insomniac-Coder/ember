#$ test: compile-fail
#$ rules: FN-5
#$ error[E0900]: a default that reads an earlier parameter is not implemented yet
# `[FN-5]` allows a default to read an earlier parameter; that half is not
# built yet and says so.

fn reads(n: int, m: int = n * 2) -> int:
    return n + m

fn main():
    println(reads(1))
