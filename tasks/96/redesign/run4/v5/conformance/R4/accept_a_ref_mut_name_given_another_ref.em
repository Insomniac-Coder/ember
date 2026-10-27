#$ test: run-pass
#$ rules: R4, R2
#$ stdout: 20 30
## R2, verdict 42: a plain `ref mut` name stands for its place until it is given another ref, with
## `ref mut` written again; `r = 20` writes `a`, `r = 30` writes `b` (AN-28).

fn main():
    a = 1
    b = 2
    r = ref mut a
    r = 20
    r = ref mut b
    r = 30
    println(a, b)
