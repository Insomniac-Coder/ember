#$ test: run-pass
#$ rules: FN-1, LT-1, BRW-12
#$ stdout: 0.5 7
## design rules: R4, R6, R2
## R4, verdict 34: `ref mut pos: MutSpan[float]` writes the caller's elements; the call says `ref mut`
## before the span made in the call. `first_even` gives back an element as a place (R6), and
## `r = 7` writes `a[1]` (AN-11).

fn integrate(ref mut pos: MutSpan[float], vel: Span[float], dt: float):
    for i in 0..pos.len():
        pos[i] += vel[i] * dt

fn first_even(ref mut xs: MutSpan[int]) -> ref mut int:
    for i in 0..xs.len():
        if xs[i] % 2 == 0:
            return ref mut xs[i]
    xs[0] = 0
    return ref mut xs[0]

fn main():
    p = [0.0, 1.0]
    v = [1.0, 1.0]
    integrate(ref mut p.as_mut_span(), v.as_span(), 0.5)
    a = [1, 2, 3]
    r = ref mut first_even(ref mut a.as_mut_span())
    r = 7
    println(p[0], a[1])
