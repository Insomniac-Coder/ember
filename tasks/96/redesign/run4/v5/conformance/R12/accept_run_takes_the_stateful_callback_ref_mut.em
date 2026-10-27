#$ test: run-pass
#$ rules: BRW-14, BRW-15
#$ stdout: ann 1
#$ stdout: ann 2
#$ stdout: 0
## design rules: R12, R4, R3
## R12 (rules.md lines 681-683, 776-781), R4: "such a function value is refused where it is given to a
## parameter that only reads (`run(cb)` with `fn run(f: fn()):`, E3023; write `ref mut f: fn()` and
## `run(ref mut cb)`)". The callback's own copy of `score` and its own `name` are its state, kept across
## the two calls, and `main`'s `score` stays 0. AN-54's NEW program, run twice; the nearest allowed
## program to reject_a_stateful_callback_given_to_a_read_only_parameter.

fn run(ref mut f: fn()):
    f()

fn main():
    name = String("ann")
    score = 0
    cb = fn() owns name, copy of score:
        score += 1
        println(name, score)
    run(ref mut cb)
    run(ref mut cb)
    println(score)
