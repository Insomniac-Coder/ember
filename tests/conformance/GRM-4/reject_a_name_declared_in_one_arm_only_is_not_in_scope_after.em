#$ test: compile-fail
#$ rules: GRM-4, CTL-10
## `[GRM-4]` — only a name declared in every arm is hoisted: one declared in one arm of the branch is
## not in scope after it.

fn main():
    if true:
        v = 1
    else:
        w = 2
    println(v)    #$ error[E1010]: cannot find `v` in this scope
