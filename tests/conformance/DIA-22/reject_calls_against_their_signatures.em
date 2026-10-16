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
    fn scale(self, k: int) -> int:
        return self.w * k

fn main():
    println(area(1))    #$ error[E2020]: `area` takes 2 arguments, found 1
    println(area(1, "x"))    #$ error[E2020]: expected `i64`, found `str`
    r = Rect(2)
    println(r.scale(1, 2))    #$ error[E2020]: `scale` takes 1 argument, found 2
    println(r.scale("k"))    #$ error[E2020]: expected `i64`, found `str`
    println(area(w=1, depth=2))    #$ error[E2020]: `area` has no parameter `depth`
