#$ test: compile-fail
#$ rules: OWN-9, GRM-15
# `[OWN-9]` (verdict 145) — `f = owns flag` hands the `bool` over, so `flag` ends there.

fn main():
    flag = true
    f = owns flag
    println(f)
    println(flag)  #$ error[E3040]
