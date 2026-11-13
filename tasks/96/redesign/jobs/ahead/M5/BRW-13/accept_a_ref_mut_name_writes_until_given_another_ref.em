# Clause BRW-13.3: "A `ref mut` name stands for the place it was made from until it is given another
# ref: `r = ref mut a` then `r = 20` writes `a`, and `r = ref mut b` points `r` at `b`, after which
# `r = 30` writes `b`."
#$ test: run-pass
#$ rules: BRW-13, TYP-14
#$ stdout: 20 30
#$ stdout: ab cd!

fn main():
    a = 1
    b = 2
    r = ref mut a
    r = 20
    r = ref mut b
    r = 30
    println(a, b)
    first = String("ab")
    second = String("cd")
    s = ref mut first
    s = ref mut second
    s += "!"
    println(first, second)
