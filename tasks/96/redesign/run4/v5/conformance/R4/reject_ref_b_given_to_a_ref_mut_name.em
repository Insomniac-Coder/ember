#$ test: compile-fail
#$ rules: R4, R2
#$ help: r = ref mut b
## R2, verdict 42: a `ref mut` name is given another ref only with `ref mut`; `ref b` only reads `b`
## (AN-28).

fn main():
    a = 1
    b = 2
    r = ref mut a
    r = 20
    r = ref b    #$ error[E3023]: `r` may change what it refers to, and `ref b` only reads `b`; to point `r` at `b` write `r = ref mut b`, or for a read-only name `rb = ref b`
    println(a, b)
