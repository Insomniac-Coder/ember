#$ test: compile-fail
#$ rules: STR-5
# `@no_derive(Clone)` opts out of the implicit `Clone`.

@no_derive(Clone)
struct Secret:
    key: String

fn main():
    s = Secret(key="k")
    t = s.clone()    #$ error[E1010]: `Secret` has no method named `clone`
