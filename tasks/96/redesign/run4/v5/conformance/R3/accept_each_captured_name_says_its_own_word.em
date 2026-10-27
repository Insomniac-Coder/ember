#$ test: run-pass
#$ rules: OWN-8
#$ stdout: ann 3
## design rules: R3
## R3: the nearest program that is allowed: each captured name has its own word.

fn main():
    name = String("ann")
    score = 3
    show = fn() copy of name, copy of score: println(name, score)
    show()
