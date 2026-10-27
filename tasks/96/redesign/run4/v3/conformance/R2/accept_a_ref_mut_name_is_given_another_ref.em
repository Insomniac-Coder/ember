#$ test: run-pass
#$ rules: R2
#$ stdout: 20 30
## R2: a `ref mut` name stands for the place it was made from until it is given another ref:
## `r = 20` writes `a`, `r = ref mut b` points `r` at `b`, and `r = 30` then writes `b` (verdict 42;
## AN-28's NEW, which prints `20 30`).

fn main():
    a = 1
    b = 2
    r = ref mut a
    r = 20
    r = ref mut b
    r = 30
    println(a, b)
