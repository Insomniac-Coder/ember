#$ test: run-pass
#$ rules: BRW-14
#$ stdout: 3 2 1 0
# `[BRW-14]` - a parameter keeps what the caller gave it; the local of your own does what Python's
# rebinding of the name does.

fn countdown(n: int):
    new_n = n
    while new_n > 0:
        print(new_n, "")
        new_n -= 1
    println(new_n)

fn main():
    countdown(3)
