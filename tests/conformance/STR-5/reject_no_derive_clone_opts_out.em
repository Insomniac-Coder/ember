#$ test: compile-fail
#$ rules: STR-5, ATT-6
# `@no_derive(Clone)` opts out of the implicit `Clone`. Opting out of `Eq` or
# `Debug` is not built, and an attribute is never accepted and ignored.

@no_derive(Clone)
struct Secret:
    key: String

@no_derive(Eq)    #$ error[E0900]: `@no_derive(Eq)` is not implemented yet
struct Plain:
    n: int

fn main():
    s = Secret(key="k")
    t = s.clone()    #$ error[E1010]: `Secret` has no method named `clone`
    println(Plain(n=1).n)
