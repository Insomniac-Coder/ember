#$ test: compile-fail
#$ rules: DIA-22
# `[DIA-22]` — a diagnostic a callee's signature causes is reported at the call, in the caller's
# code, with the callee's declaration as a secondary span: the function for a count of arguments or
# an unknown name, the parameter for an argument of the wrong type or a missing one (the milestone
# `a_call_diagnostic_points_at_the_callee_declaration` reads the secondary spans).

fn area(w: int, h: int) -> int:
    return w * h

struct Rect:
    w: int
    fn scale(k: int) -> int:
        return this.w * k

fn main():
    println(area(1))    #$ error[E2021]: `area(1)` gives nothing for `h`, which has no default: `fn area(w: int, h: int) -> int`
    println(area(1, "x"))    #$ error[E2020]: expected `i64`, found `StrView`
    r = Rect(2)
    println(r.scale(1, 2))    #$ error[E2021]: `r.scale(1, 2)` gives 2 arguments, and `scale` takes 1: `fn scale(k: int) -> int`
    println(r.scale("k"))    #$ error[E2020]: expected `i64`, found `StrView`
    println(area(w=1, depth=2))    #$ error[E2021]: `area(w=1, depth=2)` names `depth`, and `area` has no parameter of that name: `fn area(w: int, h: int) -> int`
