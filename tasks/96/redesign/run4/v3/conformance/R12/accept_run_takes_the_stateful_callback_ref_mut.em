#$ test: run-pass
#$ rules: R12, R4, R3
#$ stdout: ann 1
#$ stdout: ann 2
#$ stdout: 0
## R12 (rules.md lines 635-636, 641-646), R4: `run` takes the callback `ref mut` and the call says so; the
## callback's own copy of `score` and its own `name` are its state, kept across the two calls, and
## `main`'s `score` stays 0. AN-54's NEW program, run twice.

fn run(ref mut f: fn()):
    f()

fn main():
    name = String("ann")
    score = 0
    cb = fn() with owns name, copy of score:
        score += 1
        println(name, score)
    run(ref mut cb)
    run(ref mut cb)
    println(score)
