#$ test: compile-fail
#$ rules: R4, R5
#$ help: fn integrate(ref mut pos: MutSpan[float], vel: Span[float], dt: float):
## R4, verdict 34: a `MutSpan` parameter that writes the caller's elements says `ref mut`; a plain
## `pos: MutSpan[float]` only reads them, and the call to it takes no word (AN-11, the spec's own
## `integrate`).

fn integrate(pos: MutSpan[float], vel: Span[float], dt: float):
    for i in 0..pos.len():
        pos[i] += vel[i] * dt    #$ error[E3023]: `pos[i] += vel[i] * dt` (line 10) writes an element of the caller's span, but `pos` only reads it; write `fn integrate(ref mut pos: MutSpan[float], vel: Span[float], dt: float):` and call it `integrate(ref mut p.as_mut_span(), v.as_span(), 0.5)`

fn main():
    p = [0.0, 1.0]
    v = [1.0, 1.0]
    integrate(p.as_mut_span(), v.as_span(), 0.5)
    println(p[0])
