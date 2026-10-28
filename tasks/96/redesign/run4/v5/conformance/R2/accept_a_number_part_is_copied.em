#$ test: run-pass
#$ rules: BRW-12
#$ stdout: 2 9
## design rules: R2
## R2: a part with no fields of its own (a number) is copied, as Python binds a number: `c` is its own
## number, so changing `counts[0]` while `c` is used is allowed and `c` may change.

fn main():
    counts = [1, 2]
    c = counts[0]
    counts[0] = 9
    c += 1
    println(c, counts[0])
