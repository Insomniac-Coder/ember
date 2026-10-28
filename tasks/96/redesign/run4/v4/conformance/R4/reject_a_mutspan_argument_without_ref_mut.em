#$ test: compile-fail
#$ rules: R4
#$ help: integrate(ref mut p.as_mut_span(), v.as_span(), 0.5)
## R4, verdict 3: a `MutSpan` argument says `ref mut` at the call; before a `MutSpan` the word names
## its elements, so it stands before a span made in the call (AN-11).

fn integrate(ref mut pos: MutSpan[float], vel: Span[float], dt: float):
    for i in 0..pos.len():
        pos[i] += vel[i] * dt

fn main():
    p = [0.0, 1.0]
    v = [1.0, 1.0]
    integrate(p.as_mut_span(), v.as_span(), 0.5)    #$ error[E3028]: `integrate` changes the elements of `p` (its first line says `ref mut pos: MutSpan[float]`); say so at the call: `integrate(ref mut p.as_mut_span(), v.as_span(), 0.5)`
    println(p[0])
