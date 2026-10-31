#$ test: compile-fail
#$ rules: TYP-24, TYP-17
# D-431 — two bounds offering one name are `E2070`, with the named form as
# the help; naming an interface the bounds do not include is `E2040`.

interface A:
    fn m() -> int

interface B:
    fn m() -> int

interface D:
    fn m() -> int

[C: needs (A, B)]
fn both(c: C) -> int:
    return c.m()    #$ error[E2070]: `m` is offered by both `A` and `B`
    #$ help: name the one to call: `B.m(…)` with the receiver first

[C: needs A]
fn missing(c: C) -> int:
    return D.m(c)    #$ error[E2040]: `C` has no `m` from `D`; its bounds do not include it

fn main():
    println(0)
