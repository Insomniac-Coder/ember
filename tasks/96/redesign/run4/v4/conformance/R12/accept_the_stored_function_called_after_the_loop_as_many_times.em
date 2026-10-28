#$ test: run-pass
#$ rules: R12, R8
#$ stdout: 1
#$ stdout: 2
#$ stdout: 4
## R8 (c), R12 (rules.md lines 669-672): the program the E3020 message of the nearest refused one leads
## to: the walk of `xs` ends, then the stored function runs as many times as the loop went round.

fn main():
    xs = [1, 2]
    hs: Array[fn() changes xs] = []
    hs.push(fn() changes xs: xs.push(0))
    n = xs.len()
    for x in xs:
        println(x)
    for k in 0..n:
        hs[0]()
    println(xs.len())
