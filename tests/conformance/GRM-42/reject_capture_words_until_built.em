#$ test: compile-fail
#$ rules: GRM-42, CLI-19, GRM-15
## `[GRM-42]` — a lambda names what it keeps after its brackets with a word before each name. What
## the words mean is not built by this compiler yet, so they are `E0900` (`[CLI-19]`); `with` before
## them is `E0100`, and `ref mut`, which is no capture word, `E0120`. (`h = owns flag` hands the
## `bool` over, as for a number: verdict 145, `[OWN-9]`.)

fn main():
    score = 3
    flag = true
    f = fn() copy of score => score + 1    #$ error[E0900]: a function's capture words are not built by this compiler yet
    g = fn() with ref mut score => score    #$ error[E0100]: a function's captures follow its brackets with no `with`
                                            #$ error[E0120]: remove `ref mut score`: it is not a capture word
    h = owns flag
    println(f(), g(), h)
