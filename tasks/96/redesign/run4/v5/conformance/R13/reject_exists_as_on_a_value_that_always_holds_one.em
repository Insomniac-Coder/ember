#$ test: compile-fail
#$ rules: GRM-19
#$ help: to name it write `v = n` in place of the `if` line, and use `v` in its block, moved out one level
## design rules: R13
## R13: the left side must be able to hold nothing, an `Option` or a `weakref` link; otherwise the
## block would always run, which is refused (E2036; `V4/rules.md` section 3's message).

fn main():
    n = 5
    if n exists as v:          #$ error[E2036]: `n` is an `int`, which always holds a value, so this `if` would always run
        println(v)
