#$ test: compile-fail
#$ rules: R12, R4
#$ help: fn run(ref mut f: fn()):
## R12 (rules.md lines 766-771): a call may change the function value's own state, so "a function value
## that changes its own state is refused where it is given to a parameter that only reads" (E3023).
## AN-54 part 6; rules.md section 3's line 1046 message, word for word.

fn run(f: fn()):
    f()

fn main():
    score = 0
    cb = fn() with copy of score:
        score += 1
        println(score)
    run(cb)    #$ error[E3023]: `cb` changes its own copy of `score` each time it runs, and `run` takes it read-only; write `fn run(ref mut f: fn()):` and call `run(ref mut cb)`
    println(score)
