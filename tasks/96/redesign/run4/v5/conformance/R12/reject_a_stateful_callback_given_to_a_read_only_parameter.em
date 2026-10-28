#$ test: compile-fail
#$ rules: BRW-14, BRW-15
#$ help: write `fn run(ref mut f: fn()):` and call `run(ref mut cb)`
## design rules: R12, R4
## R12 (rules.md lines 776-781): "A call through a stored function value whose functions may change their
## own state is a change of the place that holds it ... and such a function value is refused where it is
## given to a parameter that only reads (`run(cb)` with `fn run(f: fn()):`, E3023)". AN-54 part 6;
## rules.md section 3's line 1042 message, word for word. The nearest allowed program is
## accept_run_takes_the_stateful_callback_ref_mut.

fn run(f: fn()):
    f()

fn main():
    score = 0
    cb = fn() copy of score:
        score += 1
        println(score)
    run(cb)    #$ error[E3023]: `cb` changes its own copy of `score` each time it runs, and `run` takes it read-only; write `fn run(ref mut f: fn()):` and call `run(ref mut cb)`
    println(score)
