#$ test: run-pass
#$ rules: FN-2, FN-2a
#$ stdout: ab
#$ stdout: ab
#$ stdout: 2
# `[FN-2]` — a parameter with no word reads. A callee that needs a value of its own takes it `owns`,
# and the caller passes `owns copy of x` to keep its own.

fn read(s: String) -> int:
    return s.len()

fn keep(owns s: String):
    println(s)

fn main():
    s = String("ab")
    keep(owns copy of s)
    println(s)
    println(read(s))
