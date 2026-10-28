#$ test: compile-fail
#$ rules: BRW-14, BRW-15
#$ help: rb = ref mut b
## design rules: R4, R2
## R2, verdict 42: a name written `const ref mut` is never given another ref; `const` says the name
## itself stays as it is (AN-28).

fn main():
    a = 1
    b = 2
    r = const ref mut a
    r = 20
    r = ref mut b    #$ error[E3023]: `r` is `const ref mut a`, so it always refers to `a`; to refer to `b` use a new name, `rb = ref mut b`, or make `r` with plain `ref mut`: `r = ref mut a`
    println(a, b)
