#$ test: run-pass
#$ rules: DIA-24
#$ stdout: 1 2 3 4
# `[DIA-24]` — suggestions are offered only for a name nothing declares: names one apart from each
# other (`count`, `counts`, `Count`) are all found, with nothing suggested.

struct Count:
    n: int

fn main():
    count = 1
    counts = [2, 3]
    c = Count(4)
    println(count, counts[0], counts[1], c.n)
