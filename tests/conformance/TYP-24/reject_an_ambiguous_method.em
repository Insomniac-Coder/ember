#$ test: compile-fail
#$ rules: TYP-24
# `[TYP-24]` — a method name two interfaces supply for one type is `E2070`
# at a call that does not say which; `I.m(recv)` on a type that does not
# implement `I` names what is missing.

interface A:
    fn m(self) -> int

interface B:
    fn m(self) -> int

struct S:
    n: int

struct T:
    n: int

extend S implements A:
    fn m(self) -> int:
        return 1

extend S implements B:
    fn m(self) -> int:
        return 2

fn main():
    s = S(n=0)
    t = T(n=0)
    println(s.m())          #$ error[E2070]: `m` is offered by both
    println(A.m(t))         #$ error[E2040]: `T` has no `m` from `A`
