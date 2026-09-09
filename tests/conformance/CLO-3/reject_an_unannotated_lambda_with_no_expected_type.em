#$ test: compile-fail
#$ rules: CLO-3, TYP-23
## "a lambda with unannotated parameters in a context without an expected type
## is `E2061`".

fn main():
    f = fn(x) => x * 2         #$ error[E2061]: cannot tell what `x` holds
    println(1)
