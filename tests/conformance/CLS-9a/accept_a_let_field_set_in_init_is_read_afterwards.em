#$ test: run-pass
#$ rules: CLS-9a
#$ stdout: 7
# `[CLS-9a]` — a `let` field is set in `init` and read after.

class Counter:
    let n: int

    fn init(n: int):
        this.n = n

fn main():
    c = Counter(7)
    println(c.n)
