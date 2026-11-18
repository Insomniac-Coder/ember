# Clause BRW-14.5: "every later use of the parameter in the body is written with the local, each line
# named (`[DIA-25]`), so the function does what Python's rebinding of the name does."
#$ test: run-pass
#$ rules: BRW-14, DIA-25
#$ stdout: counting
#$ stdout: tick
#$ stdout: tick
#$ stdout: 0
#$ stdout: 2
# The two programs of BRW-14.1's messages with each message's lines written, as Python would run
# the originals.

fn trim_it(line: StrView) -> int:
    new_line = line.trim()
    return new_line.len()

fn countdown(n: int):
    new_n = n
    println("counting")
    while new_n > 0:
        new_n -= 1
        println("tick")
    println(new_n)

fn main():
    countdown(2)
    println(trim_it(" ab "))
