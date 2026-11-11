#$ test: run-pass
#$ rules: DIA-2
#$ stdout: 1
# `[DIA-2]` — the line `help` gives (`this.n = 1`) is the one that compiles.

struct Counter:
    n: int

    fn bump():
        this.n = 1

fn main():
    c = Counter(n=0)
    c.bump()
    println(c.n)
