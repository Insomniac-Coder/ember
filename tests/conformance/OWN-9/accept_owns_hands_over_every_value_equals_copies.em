#$ test: run-pass
#$ rules: OWN-9, GRM-15
#$ stdout: true x 2.5
# `[OWN-9]` (verdict 145) — `owns` hands over every value `=` copies, as it does a number: a `bool`,
# a `char`, a `float`.

fn main():
    flag = true
    f = owns flag
    c = 'x'
    d = owns c
    v = 2.5
    w = owns v
    println(f, d, w)
