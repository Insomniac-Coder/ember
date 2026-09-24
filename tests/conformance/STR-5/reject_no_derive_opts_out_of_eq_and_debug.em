#$ test: compile-fail
#$ rules: STR-5
# `[STR-5]` — `@no_derive(Eq)` and `@no_derive(Debug)` opt out of the
# implicit interfaces; only `Eq`, `Debug` and `Clone` are implicit, so any
# other argument names nothing.

@no_derive(Eq)
struct Handle:
    id: int

@no_derive(Debug)
struct Secret:
    key: String

@no_derive(Copy)    #$ error[E0104]: `@no_derive(Copy)` names nothing implicit
struct Odd:
    n: int

fn main():
    a = Handle(id=1)
    b = Handle(id=1)
    println(a == b)    #$ error[E2040]: `Handle` does not implement `Eq`, which `==` needs
    s = Secret(key="k")
    println(s)    #$ error[E2040]: `Secret` does not implement `Debug`, which printing it needs
    println([s.key])
    secrets: Array[Secret] = [Secret(key="j")]
    println(secrets)    #$ error[E2040]: `Secret` does not implement `Debug`, which printing a `Array[Secret]` needs
    println(s.key, a.id, Odd(n=1).n)
