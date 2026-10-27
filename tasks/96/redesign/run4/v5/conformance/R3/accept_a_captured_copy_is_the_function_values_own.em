#$ test: run-pass
#$ rules: OWN-8, CLO-1
#$ stdout: 1
#$ 5
## design rules: R3, R12
## R3, R12: `fn() copy of score` gives the function value its own number, so a later change of
## `main`'s `score` is not seen by it (row 6; verdict 83's spelling, with no `with`; verdict 84: `copy of`
## copies, and the giver keeps its own, for every type).

fn main():
    score = 1
    show = fn() copy of score: println(score)
    score = 5
    show()
    println(score)
