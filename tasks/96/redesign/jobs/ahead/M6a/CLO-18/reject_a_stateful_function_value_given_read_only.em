# Clause CLO-18.2: "Such a value given to a parameter that only reads is `E3023` too"; `[FN-2]`:
# "one whose functions may change their own state is given `ref mut h`, since the call changes it
# (`[CLO-18]`, `E3023` without the word"
#$ test: compile-fail
#$ rules: CLO-18, FN-2, CLO-14
# Both messages are the spec's, with this file's names. Capture words are M8's.

fn run(f: fn()):
    f()

[F: needs fn()]
fn apply(owns f: F):
    f()

fn main():
    score = 0
    cb = fn() copy of score: score += 1
    run(cb)    #$ error[E3023]: `cb` changes its own copy of `score` each time it runs, and `run` takes it read-only; write `fn run(ref mut f: fn()):` and call `run(ref mut cb)`
    n = 0
    counter = fn() copy of n: n += 1
    apply(counter)    #$ error[E3023]: `counter` changes its own `n` each time it runs, and `apply` runs it; say so at the call: `apply(ref mut counter)`
