#$ test: run-pass
#$ rules: R3, R12
#$ stdout: 1
#$ 5
## R3, R12: the nearest program that is allowed: `with copy of score` gives the function value its
## own number, so a later change of `main`'s `score` is not seen by it (row 6).

fn main():
    score = 1
    show = fn() with copy of score: println(score)
    score = 5
    show()
    println(score)
